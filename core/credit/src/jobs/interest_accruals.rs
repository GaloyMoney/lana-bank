use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::OutboxEventMarker;

use crate::{credit_facility::CreditFacilities, event::CoreCreditEvent, ledger::*, primitives::*};

#[derive(Clone, Serialize, Deserialize)]
pub struct InterestAccrualJobConfig<Perms, E> {
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> JobConfig for InterestAccrualJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = InterestAccrualJobInitializer<Perms, E>;
}

pub struct InterestAccrualJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    ledger: CreditLedger,
    credit_facilities: CreditFacilities<Perms, E>,
    jobs: Jobs,
}

impl<Perms, E> InterestAccrualJobInitializer<Perms, E>
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

const INTEREST_ACCRUAL_JOB: JobType = JobType::new("interest-accrual");
impl<Perms, E> JobInitializer for InterestAccrualJobInitializer<Perms, E>
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
        INTEREST_ACCRUAL_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(InterestAccrualJobRunner::<Perms, E> {
            config: job.config()?,
            credit_facilities: self.credit_facilities.clone(),
            ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
        }))
    }
}

pub struct InterestAccrualJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    config: InterestAccrualJobConfig<Perms, E>,
    credit_facilities: CreditFacilities<Perms, E>,
    ledger: CreditLedger,
    jobs: Jobs,
}

#[async_trait]
impl<Perms, E> JobRunner for InterestAccrualJobRunner<Perms, E>
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
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
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
                    super::interest_accrual_cycles::InterestAccrualCycleJobConfig::<Perms, E> {
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
