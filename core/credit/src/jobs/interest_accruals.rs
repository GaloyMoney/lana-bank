use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::error::JobRunError;
use job::*;
use outbox::OutboxEventMarker;

use crate::{credit_facility::CreditFacilities, event::CoreCreditEvent, ledger::*, primitives::*};

#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityJobConfig<Perms, E> {
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> JobConfig for CreditFacilityJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = CreditFacilityProcessingJobInitializer<Perms, E>;
}

pub struct CreditFacilityProcessingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    ledger: CreditLedger,
    credit_facilities: CreditFacilities<Perms, E>,
    jobs: Jobs,
}

impl<Perms, E> CreditFacilityProcessingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        ledger: &CreditLedger,
        credit_facilities: &CreditFacilities<Perms, E>,
        jobs: &Jobs,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            credit_facilities: credit_facilities.clone(),
            jobs: jobs.clone(),
        }
    }
}

const CREDIT_FACILITY_INTEREST_ACCRUAL_PROCESSING_JOB: JobType =
    JobType::new("credit-facility-interest-accrual-processing");
impl<Perms, E> JobInitializer for CreditFacilityProcessingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_INTEREST_ACCRUAL_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner::<Perms, E> {
            config: job.config()?,
            credit_facilities: self.credit_facilities.clone(),
            ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
        }))
    }
}

pub struct CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    config: CreditFacilityJobConfig<Perms, E>,
    credit_facilities: CreditFacilities<Perms, E>,
    ledger: CreditLedger,
    jobs: Jobs,
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(
        name = "credit-facility.interest-accruals.job",
        skip(self, current_job),
        fields(attempt)
    )]
    async fn run(&self, current_job: CurrentJob) -> Result<JobCompletion, JobRunError> {
        let span = tracing::Span::current();
        span.record("attempt", current_job.attempt());

        let mut db = self.credit_facilities.begin_op().await?;

        let crate::ConfirmedAccrual {
            accrual: interest_accrual,
            next_period: next_accrual_period,
            accrual_idx,
            accrued_count,
        } = self
            .credit_facilities
            .confirm_interest_accrual_in_op(&mut db, self.config.credit_facility_id)
            .await?;

        if let Some(period) = next_accrual_period {
            self.ledger
                .record_interest_accrual(db, interest_accrual)
                .await?;
            Ok(JobCompletion::RescheduleAt(period.end))
        } else {
            self.jobs
                .create_and_spawn_in_op(
                    &mut db,
                    uuid::Uuid::new_v4(),
                    super::interest_accrual_cycles::CreditFacilityJobConfig::<Perms, E> {
                        credit_facility_id: self.config.credit_facility_id,
                        _phantom: std::marker::PhantomData,
                    },
                )
                .await?;
            self.ledger
                .record_interest_accrual(db, interest_accrual)
                .await?;

            println!(
                "All ({:?}) accruals completed for {:?} of {:?}",
                accrued_count, accrual_idx, self.config.credit_facility_id
            );
            Ok(JobCompletion::Complete)
        }
    }
}
