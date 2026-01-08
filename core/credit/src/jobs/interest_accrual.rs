use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::out::OutboxEventMarker;

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId,
    credit_facility::{CreditFacilities, interest_accrual_cycle::NewInterestAccrualCycleData},
    ledger::*,
    obligation::Obligations,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum InterestAccrualPhase {
    /// Accrue interest for the current period within a cycle
    AccrueInterest,
    /// Complete the current cycle and potentially start a new one
    CompleteCycle,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InterestAccrualJobConfig<Perms, E> {
    pub credit_facility_id: CreditFacilityId,
    pub phase: InterestAccrualPhase,
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
    type Initializer = InterestAccrualJobInit<Perms, E>;
}

pub struct InterestAccrualJobInit<Perms, E>
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

impl<Perms, E> InterestAccrualJobInit<Perms, E>
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

const INTEREST_ACCRUAL_JOB: JobType = JobType::new("task.interest-accrual");

impl<Perms, E> JobInitializer for InterestAccrualJobInit<Perms, E>
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
            obligations: self.obligations.clone(),
            credit_facilities: self.credit_facilities.clone(),
            ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
            audit: self.audit.clone(),
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
    obligations: Obligations<Perms, E>,
    credit_facilities: CreditFacilities<Perms, E>,
    ledger: CreditLedger,
    jobs: Jobs,
    audit: Perms::Audit,
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
        match &self.config.phase {
            InterestAccrualPhase::AccrueInterest => self.run_accrue_interest().await,
            InterestAccrualPhase::CompleteCycle => self.run_complete_cycle().await,
        }
    }
}

impl<Perms, E> InterestAccrualJobRunner<Perms, E>
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
    async fn run_accrue_interest(&self) -> Result<JobCompletion, Box<dyn std::error::Error>> {
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

        self.ledger
            .record_interest_accrual(
                &mut db,
                interest_accrual,
                core_accounting::LedgerTransactionInitiator::System,
            )
            .await?;

        if let Some(period) = next_accrual_period {
            Ok(JobCompletion::RescheduleAtWithOp(db, period.end))
        } else {
            // All accruals in this cycle complete - spawn cycle completion phase
            self.jobs
                .create_and_spawn_in_op(
                    &mut db,
                    uuid::Uuid::new_v4(),
                    InterestAccrualJobConfig::<Perms, E> {
                        credit_facility_id: self.config.credit_facility_id,
                        phase: InterestAccrualPhase::CompleteCycle,
                        _phantom: std::marker::PhantomData,
                    },
                )
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

    async fn run_complete_cycle(&self) -> Result<JobCompletion, Box<dyn std::error::Error>> {
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

        let crate::CompletedAccrualCycle {
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
                    InterestAccrualJobConfig::<Perms, E> {
                        credit_facility_id: self.config.credit_facility_id,
                        phase: InterestAccrualPhase::AccrueInterest,
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

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
