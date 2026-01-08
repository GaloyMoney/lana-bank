//! Interest Accrual Job - State Machine
//!
//! This job manages the complete interest accrual lifecycle for a credit facility.
//! It operates as a state machine with the following states and transitions:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                         InterestAccrualState                             │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  AccruePeriod                                                            │
//! │    • Calculate interest for current accrual period                       │
//! │    • Record accrual to ledger                                            │
//! │    → more periods remaining: RescheduleAt(next_period.end)               │
//! │    → cycle complete: transition to AwaitObligationsSync                  │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  AwaitObligationsSync                                                    │
//! │    • Wait for all facility obligations to reach current status           │
//! │    • Required before cycle completion to ensure consistent state         │
//! │    → not ready: RescheduleIn(5 min)                                      │
//! │    → ready: transition to CompleteCycle                                  │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  CompleteCycle                                                           │
//! │    • Finalize the interest accrual cycle                                 │
//! │    • Create interest obligation (if accrued amount > 0)                  │
//! │    • Record cycle completion to ledger                                   │
//! │    → new cycle exists: spawn new job in AccruePeriod state               │
//! │    → facility matured: complete                                          │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```

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

/// State machine states for the interest accrual job.
///
/// Each state represents a discrete domain process in the interest accrual lifecycle.
/// This is stored in the job's execution_state and persists across reschedules.
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum InterestAccrualState {
    /// Calculate and record interest for the current accrual period.
    ///
    /// This state handles individual period accruals within a cycle.
    /// A cycle may contain multiple periods (e.g., daily accruals within a monthly cycle).
    #[default]
    AccruePeriod,

    /// Wait for facility obligations to be synchronized.
    ///
    /// Before completing a cycle, we must ensure all obligations have their
    /// status updated. This prevents race conditions where an obligation's
    /// status change could affect the cycle completion logic.
    AwaitObligationsSync,

    /// Complete the current interest accrual cycle.
    ///
    /// This finalizes the cycle by:
    /// - Creating an interest obligation for the total accrued amount
    /// - Recording the cycle completion to the ledger
    /// - Initiating the next cycle if the facility hasn't matured
    CompleteCycle,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InterestAccrualJobConfig<Perms, E> {
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

struct InterestAccrualJobRunner<Perms, E>
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
    #[tracing::instrument(
        name = "interest_accrual.run",
        skip(self, current_job),
        fields(credit_facility_id = %self.config.credit_facility_id)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<InterestAccrualState>()?
            .unwrap_or_default();

        tracing::debug!(?state, "Executing interest accrual state");

        match state {
            InterestAccrualState::AccruePeriod => self.accrue_period(current_job).await,
            InterestAccrualState::AwaitObligationsSync => {
                self.await_obligations_sync(current_job).await
            }
            InterestAccrualState::CompleteCycle => self.complete_cycle(current_job).await,
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
    /// State: AccruePeriod
    ///
    /// Calculates interest for the current period and records it to the ledger.
    /// Transitions:
    /// - If more periods remain in the cycle: reschedule at next period end
    /// - If cycle is complete: transition to AwaitObligationsSync
    async fn accrue_period(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut db = self.credit_facilities.begin_op().await?;

        let crate::ConfirmedAccrual {
            accrual: interest_accrual,
            next_period,
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

        match next_period {
            Some(period) => {
                tracing::debug!(
                    accrual_idx = %accrual_idx,
                    next_period_end = %period.end,
                    "Period accrued, scheduling next period"
                );
                Ok(JobCompletion::RescheduleAtWithOp(db, period.end))
            }
            None => {
                tracing::info!(
                    accrued_count = %accrued_count,
                    accrual_idx = %accrual_idx,
                    "All periods accrued, transitioning to await obligations sync"
                );
                current_job
                    .update_execution_state_in_op(
                        &mut db,
                        &InterestAccrualState::AwaitObligationsSync,
                    )
                    .await?;
                db.commit().await?;
                self.await_obligations_sync(current_job).await
            }
        }
    }

    /// State: AwaitObligationsSync
    ///
    /// Waits for all facility obligations to have their status updated.
    /// This is required before cycle completion to ensure consistent state.
    /// Transitions:
    /// - If obligations not synced: reschedule in 5 minutes
    /// - If obligations synced: transition to CompleteCycle
    async fn await_obligations_sync(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let obligations_synced = self
            .obligations
            .check_facility_obligations_status_updated(self.config.credit_facility_id)
            .await?;

        if !obligations_synced {
            tracing::debug!("Obligations not yet synced, rescheduling");
            return Ok(JobCompletion::RescheduleIn(std::time::Duration::from_secs(
                5 * 60,
            )));
        }

        tracing::debug!("Obligations synced, transitioning to complete cycle");
        current_job
            .update_execution_state(&InterestAccrualState::CompleteCycle)
            .await?;
        self.complete_cycle(current_job).await
    }

    /// State: CompleteCycle
    ///
    /// Finalizes the interest accrual cycle:
    /// - Records audit entry
    /// - Completes the cycle (creates interest obligation if amount > 0)
    /// - Records cycle completion to ledger
    /// - Spawns new job for next cycle if facility hasn't matured
    async fn complete_cycle(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
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

        self.ledger
            .record_interest_accrual_cycle(
                &mut op,
                facility_accrual_cycle_data,
                core_accounting::LedgerTransactionInitiator::System,
            )
            .await?;

        match new_cycle_data {
            Some(NewInterestAccrualCycleData {
                id: new_accrual_cycle_id,
                first_accrual_end_date,
            }) => {
                tracing::info!(
                    new_cycle_id = %new_accrual_cycle_id,
                    first_accrual_end = %first_accrual_end_date,
                    "Cycle completed, starting new cycle"
                );
                self.jobs
                    .create_and_spawn_at_in_op(
                        &mut op,
                        new_accrual_cycle_id,
                        InterestAccrualJobConfig::<Perms, E> {
                            credit_facility_id: self.config.credit_facility_id,
                            _phantom: std::marker::PhantomData,
                        },
                        first_accrual_end_date,
                    )
                    .await?;
            }
            None => {
                tracing::info!("All interest accrual cycles completed - facility matured");
            }
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
