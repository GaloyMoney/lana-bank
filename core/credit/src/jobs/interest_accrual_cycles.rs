use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilityRepo,
    interest_accruals,
    ledger::*,
    obligation::{Obligation, Obligations},
    CoreCreditAction, CoreCreditError, CoreCreditEvent, CoreCreditObject, CreditFacilityId,
    InterestAccrualCycleId,
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
    obligations: Obligations<Perms, E>,
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
        obligations: &Obligations<Perms, E>,
        credit_facility_repo: &CreditFacilityRepo<E>,
        jobs: &Jobs,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            obligations: obligations.clone(),
            credit_facility_repo: credit_facility_repo.clone(),
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
            obligations: self.obligations.clone(),
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
    obligations: Obligations<Perms, E>,
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
    ) -> Result<(Obligation, Option<(InterestAccrualCycleId, DateTime<Utc>)>), CoreCreditError>
    {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(self.config.credit_facility_id)
            .await?;

        let new_obligation = if let es_entity::Idempotent::Executed(new_obligation) =
            credit_facility.record_interest_accrual_cycle(audit_info.clone())?
        {
            new_obligation
        } else {
            unreachable!("Should not be possible");
        };

        let obligation = self
            .obligations
            .create_with_jobs_in_op(db, new_obligation)
            .await?;
        credit_facility
            .update_balance_from_obligation(
                obligation.id,
                obligation.obligation_type(),
                obligation.initial_amount,
                obligation.recorded_at,
                audit_info.clone(),
            )
            .did_execute();

        let res = credit_facility.start_interest_accrual_cycle(audit_info.clone())?;
        self.credit_facility_repo
            .update_in_op(db, &mut credit_facility)
            .await?;
        let credit_facility = credit_facility;

        let new_cycle_data = res.map(|periods| {
            let new_accrual_cycle_id = credit_facility
                .interest_accrual_cycle_in_progress()
                .expect("First accrual cycle not found")
                .id;

            (new_accrual_cycle_id, periods.accrual.end)
        });

        Ok((obligation, new_cycle_data))
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

        if !self
            .obligations
            .check_facility_obligations_status_updated(self.config.credit_facility_id)
            .await?
        {
            let now = crate::time::now();
            return Ok(JobCompletion::RescheduleAt(now + Duration::minutes(5)));
        }

        let mut db = self.credit_facility_repo.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let (obligation, new_cycle_data) = self
            .complete_interest_cycle_and_maybe_start_new_cycle(&mut db, &audit_info)
            .await?;

        if let Some((new_accrual_cycle_id, first_accrual_end_date)) = new_cycle_data {
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
        } else {
            println!(
                "All Credit Facility interest accrual cycles completed for credit_facility: {:?}",
                self.config.credit_facility_id
            );
        };

        self.ledger
            .record_interest_accrual_cycle(db, obligation)
            .await?;

        return Ok(JobCompletion::Complete);
    }
}
