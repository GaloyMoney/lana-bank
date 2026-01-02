use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::out::OutboxEventMarker;

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{
    CompletedAccrualCycle, CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId,
    credit_facility::{CreditFacilities, interest_accrual_cycle::NewInterestAccrualCycleData},
    interest_accruals,
    ledger::*,
    obligation::Obligations,
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct InterestAccrualCycleJobConfig<Perms, E> {
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for InterestAccrualCycleJobConfig<Perms, E>
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
    type Initializer = InterestAccrualCycleInit<Perms, E>;
}

pub(crate) struct InterestAccrualCycleInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    ledger: CreditLedger,
    obligations: Obligations<Perms, E>,
    credit_facilities: CreditFacilities<Perms, E>,
    jobs: Jobs,
    audit: Perms::Audit,
}

impl<Perms, E> InterestAccrualCycleInit<Perms, E>
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
        obligations: &Obligations<Perms, E>,
        credit_facilities: &CreditFacilities<Perms, E>,
        jobs: &Jobs,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            obligations: obligations.clone(),
            credit_facilities: credit_facilities.clone(),
            jobs: jobs.clone(),
            audit: audit.clone(),
        }
    }
}

// per credit facility job type
const INTEREST_ACCRUAL_CYCLE_JOB: JobType = JobType::new("task.interest-accrual-cycle");
impl<Perms, E> JobInitializer for InterestAccrualCycleInit<Perms, E>
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
        INTEREST_ACCRUAL_CYCLE_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(InterestAccrualCycleJobRunner::<Perms, E> {
            config: job.config()?,
            obligations: self.obligations.clone(),
            credit_facilities: self.credit_facilities.clone(),
            ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct InterestAccrualCycleJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    config: InterestAccrualCycleJobConfig<Perms, E>,
    obligations: Obligations<Perms, E>,
    credit_facilities: CreditFacilities<Perms, E>,
    ledger: CreditLedger,
    jobs: Jobs,
    audit: Perms::Audit,
}

#[async_trait]
impl<Perms, E> JobRunner for InterestAccrualCycleJobRunner<Perms, E>
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
        if !self
            .obligations
            .check_facility_obligations_status_updated(self.config.credit_facility_id)
            .await?
        {
            return Ok(JobCompletion::RescheduleIn(std::time::Duration::from_secs(
                5 * 60,
            )));
        }

        let mut op = self.credit_facilities.begin_op().await?;
        self.audit
            .record_system_entry_in_tx(
                &mut op,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let CompletedAccrualCycle {
            facility_accrual_cycle_data,
            new_cycle_data,
        } = self
            .credit_facilities
            .complete_interest_cycle_and_maybe_start_new_cycle(
                &mut op,
                self.config.credit_facility_id,
            )
            .await?;

        if let Some(new_cycle_data) = new_cycle_data {
            let NewInterestAccrualCycleData {
                id: new_accrual_cycle_id,
                first_accrual_end_date,
            } = new_cycle_data;

            self.jobs
                .create_and_spawn_at_in_op(
                    &mut op,
                    new_accrual_cycle_id,
                    interest_accruals::InterestAccrualJobConfig::<Perms, E> {
                        credit_facility_id: self.config.credit_facility_id,
                        _phantom: std::marker::PhantomData,
                    },
                    first_accrual_end_date,
                )
                .await?;
        } else {
            tracing::info!(
                credit_facility_id = %self.config.credit_facility_id,
                "All interest accrual cycles completed for {}",
                self.config.credit_facility_id
            );
        };

        self.ledger
            .record_interest_accrual_cycle(
                &mut op,
                facility_accrual_cycle_data,
                core_accounting::LedgerTransactionInitiator::System,
            )
            .await?;

        return Ok(JobCompletion::CompleteWithOp(op));
    }
}
