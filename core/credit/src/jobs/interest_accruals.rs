use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    interest_incurrences, ledger::*, repo::*, CoreCreditAction, CoreCreditEvent, CoreCreditObject,
    CreditFacilityError, CreditFacilityId, CreditFacilityInterestAccrual,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityJobConfig<Perms, E> {
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for CreditFacilityJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = CreditFacilityProcessingJobInitializer<Perms, E>;
}

pub struct CreditFacilityProcessingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    ledger: CreditLedger,
    credit_facility_repo: CreditFacilityRepo<E>,
    jobs: Jobs,
    audit: Perms::Audit,
}

impl<Perms, E> CreditFacilityProcessingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        ledger: &CreditLedger,
        credit_facility_repo: CreditFacilityRepo<E>,
        jobs: &Jobs,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            credit_facility_repo,
            jobs: jobs.clone(),
            audit: audit.clone(),
        }
    }
}

const CREDIT_FACILITY_INTEREST_PROCESSING_JOB: JobType =
    JobType::new("credit-facility-interest-accrual-processing");
impl<Perms, E> JobInitializer for CreditFacilityProcessingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_INTEREST_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner::<Perms, E> {
            config: job.config()?,
            credit_facility_repo: self.credit_facility_repo.clone(),
            ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CreditFacilityJobConfig<Perms, E>,
    credit_facility_repo: CreditFacilityRepo<E>,
    ledger: CreditLedger,
    jobs: Jobs,
    audit: Perms::Audit,
}

impl<Perms, E> CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[es_entity::retry_on_concurrent_modification]
    async fn record_interest_accrual(
        &self,
        db: &mut es_entity::DbOp<'_>,
        audit_info: &AuditInfo,
    ) -> Result<CreditFacilityInterestAccrual, CreditFacilityError> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(self.config.credit_facility_id)
            .await?;

        let interest_accrual = credit_facility.record_interest_accrual(audit_info.clone())?;
        self.credit_facility_repo
            .update_in_op(db, &mut credit_facility)
            .await?;

        Ok(interest_accrual)
    }
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(
        name = "credit-facility.interest-accruals.job",
        skip(self, current_job),
        fields(attempt)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let span = tracing::Span::current();
        span.record("attempt", current_job.attempt());

        let mut db = self.credit_facility_repo.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let interest_accrual = self.record_interest_accrual(&mut db, &audit_info).await?;

        let (now, mut tx) = (db.now(), db.into_tx());
        let sub_op = {
            use sqlx::Acquire;
            es_entity::DbOp::new(tx.begin().await?, now)
        };
        self.ledger
            .record_interest_accrual(sub_op, interest_accrual)
            .await?;

        let mut db = es_entity::DbOp::new(tx, now);
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id_in_tx(db.tx(), self.config.credit_facility_id)
            .await?;
        let periods = match credit_facility.start_interest_accrual(audit_info)? {
            Some(periods) => periods,
            None => {
                println!(
                    "Credit Facility interest accrual job completed for credit_facility: {:?}",
                    self.config.credit_facility_id
                );

                return Ok(JobCompletion::CompleteWithOp(db));
            }
        };

        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        let accrual_id = credit_facility
            .interest_accrual_in_progress()
            .expect("First accrual not found")
            .id;
        self.jobs
            .create_and_spawn_at_in_op(
                &mut db,
                accrual_id,
                interest_incurrences::CreditFacilityJobConfig::<Perms, E> {
                    credit_facility_id: credit_facility.id,
                    _phantom: std::marker::PhantomData,
                },
                periods.incurrence.end,
            )
            .await?;

        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        return Ok(JobCompletion::CompleteWithOp(db));
    }
}
