use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilityRepo, interest_accruals, ledger::*, CoreCreditAction,
    CoreCreditError, CoreCreditEvent, CoreCreditObject, CreditFacilityId,
    CreditFacilityInterestAccrualCycle, InterestAccrualCycleId,
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

const CREDIT_FACILITY_INTEREST_ACCRUAL_CYCLE_PROCESSING_JOB: JobType =
    JobType::new("credit-facility-interest-accrual-cycle-processing");
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
        CREDIT_FACILITY_INTEREST_ACCRUAL_CYCLE_PROCESSING_JOB
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
    async fn complete_interest_cycle_and_maybe_start_new_cycle(
        &self,
        db: &mut es_entity::DbOp<'_>,
        audit_info: &AuditInfo,
    ) -> Result<
        Option<(
            CreditFacilityInterestAccrualCycle,
            InterestAccrualCycleId,
            DateTime<Utc>,
        )>,
        CoreCreditError,
    > {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(self.config.credit_facility_id)
            .await?;

        let completed_interest_accrual_cycle =
            credit_facility.record_interest_accrual_cycle(audit_info.clone())?;
        self.credit_facility_repo
            .update_in_op(db, &mut credit_facility)
            .await?;

        let first_accrual_period_in_new_cycle =
            match credit_facility.start_interest_accrual_cycle(audit_info.clone())? {
                Some(p) => p.accrual,
                None => return Ok(None),
            };
        self.credit_facility_repo
            .update_in_op(db, &mut credit_facility)
            .await?;

        let new_accrual_cycle_id = credit_facility
            .interest_accrual_cycle_in_progress() // TODO: remove &mut from self arg for this fn
            .expect("First accrual cycle not found")
            .id;

        Ok(Some((
            completed_interest_accrual_cycle,
            new_accrual_cycle_id,
            first_accrual_period_in_new_cycle.end,
        )))
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
        name = "credit-facility.interest-accrual-cycles.job",
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

        let (completed_interest_accrual_cycle, new_accrual_cycle_id, first_accrual_end_date) =
            if let Some((
                completed_interest_accrual_cycle,
                new_accrual_cycle_id,
                first_accrual_end_date,
            )) = self
                .complete_interest_cycle_and_maybe_start_new_cycle(&mut db, &audit_info)
                .await?
            {
                (
                    completed_interest_accrual_cycle,
                    new_accrual_cycle_id,
                    first_accrual_end_date,
                )
            } else {
                println!(
                    "Credit Facility interest accrual job completed for credit_facility: {:?}",
                    self.config.credit_facility_id
                );
                return Ok(JobCompletion::CompleteWithOp(db));
            };

        self.jobs
            .create_and_spawn_at_in_op(
                &mut db,
                new_accrual_cycle_id,
                interest_accruals::CreditFacilityJobConfig::<Perms, E> {
                    credit_facility_id: self.config.credit_facility_id,
                    _phantom: std::marker::PhantomData,
                },
                first_accrual_end_date,
            )
            .await?;

        self.ledger
            .record_interest_accrual_cycle(db, completed_interest_accrual_cycle)
            .await?;

        return Ok(JobCompletion::Complete);
    }
}
