use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::out::OutboxEventMarker;

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{credit_facility::CreditFacilities, event::CoreCreditEvent, ledger::*, primitives::*};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct InterestAccrualJobConfig<Perms, E> {
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> JobConfig for InterestAccrualJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Initializer = InterestAccrualInit<Perms, E>;
}

pub struct InterestAccrualInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    ledger: CreditLedger,
    credit_facilities: CreditFacilities<Perms, E>,
    jobs: Jobs,
}

impl<Perms, E> InterestAccrualInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
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

const INTEREST_ACCRUAL_JOB: JobType = JobType::new("task.interest-accrual");
impl<Perms, E> JobInitializer for InterestAccrualInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
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
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
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
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
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
                .record_interest_accrual(&mut db, interest_accrual)
                .await?;
            Ok(JobCompletion::RescheduleAtWithOp(period.end, db))
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
                .record_interest_accrual(&mut db, interest_accrual)
                .await?;

            tracing::info!(
                accrued_count = %accrued_count,
                accrual_idx = %accrual_idx,
                credit_facility_id = %self.config.credit_facility_id,
                "All accruals completed for period"
            );
            Ok(JobCompletion::CompleteWithOp(db))
        }
    }
}
