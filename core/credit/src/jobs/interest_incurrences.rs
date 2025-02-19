use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    ledger::*, repo::*, CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityError,
    CreditFacilityId, CreditFacilityInterestIncurrence, InterestAccrualIdx, InterestPeriod,
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
    audit: Perms::Audit,
    jobs: Jobs,
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
        audit: &Perms::Audit,
        jobs: &Jobs,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            credit_facility_repo,
            audit: audit.clone(),
            jobs: jobs.clone(),
        }
    }
}

const CREDIT_FACILITY_INTEREST_INCURRENCE_PROCESSING_JOB: JobType =
    JobType::new("credit-facility-interest-incurrence-processing");
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
        CREDIT_FACILITY_INTEREST_INCURRENCE_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner::<Perms, E> {
            config: job.config()?,
            credit_facility_repo: self.credit_facility_repo.clone(),
            ledger: self.ledger.clone(),
            audit: self.audit.clone(),
            jobs: self.jobs.clone(),
        }))
    }
}

#[derive(Clone)]
struct ConfirmedIncurrence {
    incurrence: CreditFacilityInterestIncurrence,
    next_period: Option<InterestPeriod>,
    accrual_idx: InterestAccrualIdx,
}

pub struct CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CreditFacilityJobConfig<Perms, E>,
    credit_facility_repo: CreditFacilityRepo<E>,
    ledger: CreditLedger,
    audit: Perms::Audit,
    jobs: Jobs,
}

impl<Perms, E> CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[es_entity::retry_on_concurrent_modification]
    async fn confirm_interest_incurrence(
        &self,
        db: &mut es_entity::DbOp<'_>,
        audit_info: &AuditInfo,
    ) -> Result<ConfirmedIncurrence, CreditFacilityError> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(self.config.credit_facility_id)
            .await?;

        let confirmed_incurrence = {
            let outstanding = credit_facility.outstanding();
            let account_ids = credit_facility.account_ids;

            let accrual = credit_facility
                .interest_accrual_in_progress()
                .expect("Accrual in progress should exist for scheduled job");

            let interest_incurrence = accrual.record_incurrence(outstanding, audit_info.clone());

            ConfirmedIncurrence {
                incurrence: (interest_incurrence, account_ids).into(),
                next_period: accrual.next_incurrence_period(),
                accrual_idx: accrual.idx,
            }
        };

        self.credit_facility_repo
            .update_in_op(db, &mut credit_facility)
            .await?;

        Ok(confirmed_incurrence)
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
        name = "credit-facility.interest-incurrences.job",
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

        let ConfirmedIncurrence {
            incurrence: interest_incurrence,
            next_period: next_incurrence_period,
            accrual_idx,
        } = self
            .confirm_interest_incurrence(&mut db, &audit_info)
            .await?;

        let (now, mut tx) = (db.now(), db.into_tx());
        let sub_op = {
            use sqlx::Acquire;
            es_entity::DbOp::new(tx.begin().await?, now)
        };
        self.ledger
            .record_interest_incurrence(sub_op, interest_incurrence)
            .await?;

        let mut db = es_entity::DbOp::new(tx, now);
        if let Some(period) = next_incurrence_period {
            Ok(JobCompletion::RescheduleAtWithOp(db, period.end))
        } else {
            self.jobs
                .create_and_spawn_in_op(
                    &mut db,
                    uuid::Uuid::new_v4(),
                    super::interest_accruals::CreditFacilityJobConfig::<Perms, E> {
                        credit_facility_id: self.config.credit_facility_id,
                        _phantom: std::marker::PhantomData,
                    },
                )
                .await?;
            println!(
            "Credit Facility interest incurrences job completed for accrual index {:?} for credit_facility: {:?}",
            accrual_idx, self.config.credit_facility_id
        );
            Ok(JobCompletion::CompleteWithOp(db))
        }
    }
}
