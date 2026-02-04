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
use tracing::instrument;
use tracing_macros::record_error_severity;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::out::OutboxEventMarker;

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;

use crate::{
    CompletedAccrualCycle, ConfirmedAccrual, CoreCreditAction, CoreCreditCollectionAction,
    CoreCreditCollectionEvent, CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject,
    CreditFacilityId,
    credit_facility::{
        CreditFacilityRepo, error::CreditFacilityError,
        interest_accrual_cycle::NewInterestAccrualCycleData,
    },
    ledger::*,
};

use core_credit_collection::CoreCreditCollection;

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

#[derive(Serialize, Deserialize)]
pub struct InterestAccrualJobConfig<Perms, E> {
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for InterestAccrualJobConfig<Perms, E> {
    fn clone(&self) -> Self {
        Self {
            credit_facility_id: self.credit_facility_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct InterestAccrualJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    ledger: Arc<CreditLedger>,
    collections: Arc<CoreCreditCollection<Perms, E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    authz: Arc<Perms>,
}

impl<Perms, E> InterestAccrualJobInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        ledger: Arc<CreditLedger>,
        collections: Arc<CoreCreditCollection<Perms, E>>,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        authz: Arc<Perms>,
    ) -> Self {
        Self {
            ledger,
            collections,
            credit_facility_repo,
            authz,
        }
    }
}

const INTEREST_ACCRUAL_JOB: JobType = JobType::new("task.interest-accrual");

impl<Perms, E> JobInitializer for InterestAccrualJobInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = InterestAccrualJobConfig<Perms, E>;
    fn job_type(&self) -> JobType {
        INTEREST_ACCRUAL_JOB
    }

    fn init(
        &self,
        job: &Job,
        spawner: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(InterestAccrualJobRunner::<Perms, E> {
            config: job.config()?,
            collections: self.collections.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            ledger: self.ledger.clone(),
            spawner,
            authz: self.authz.clone(),
        }))
    }
}

struct InterestAccrualJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: InterestAccrualJobConfig<Perms, E>,
    collections: Arc<CoreCreditCollection<Perms, E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    ledger: Arc<CreditLedger>,
    spawner: InterestAccrualJobSpawner<Perms, E>,
    authz: Arc<Perms>,
}

#[async_trait]
impl<Perms, E> JobRunner for InterestAccrualJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
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
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
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
        let mut db = self.credit_facility_repo.begin_op().await?;

        let ConfirmedAccrual {
            accrual: interest_accrual,
            next_period,
            accrual_idx,
            accrued_count,
        } = self
            .confirm_interest_accrual_in_op(&mut db, self.config.credit_facility_id)
            .await?;

        self.ledger
            .record_interest_accrual_in_op(
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

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.confirm_interest_accrual_in_op",
        skip(self, op),
        fields(credit_facility_id = %credit_facility_id)
    )]
    async fn confirm_interest_accrual_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        credit_facility_id: CreditFacilityId,
    ) -> Result<ConfirmedAccrual, CreditFacilityError> {
        self.authz
            .audit()
            .record_system_entry_in_op(
                op,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let confirmed_accrual = {
            let account_ids = credit_facility.account_ids;
            let balances = self.ledger.get_credit_facility_balance(account_ids).await?;

            let recorded = credit_facility
                .record_accrual_on_in_progress_cycle(balances.disbursed_outstanding())?
                .expect("record_accrual always returns Executed");

            ConfirmedAccrual {
                accrual: (recorded.accrual_data, account_ids).into(),
                next_period: recorded.next_period,
                accrual_idx: recorded.accrual_idx,
                accrued_count: recorded.accrued_count,
            }
        };

        self.credit_facility_repo
            .update_in_op(op, &mut credit_facility)
            .await?;

        Ok(confirmed_accrual)
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
            .collections
            .obligations()
            .check_beneficiary_obligations_status_updated(self.config.credit_facility_id.into())
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
        let mut op = self.credit_facility_repo.begin_op().await?;

        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut op,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let CompletedAccrualCycle {
            facility_accrual_cycle_data,
            new_cycle_data,
        } = self
            .complete_interest_cycle_and_maybe_start_new_cycle_in_op(
                &mut op,
                self.config.credit_facility_id,
            )
            .await?;

        self.ledger
            .record_interest_accrual_cycle_in_op(
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
                self.spawner
                    .spawn_at_in_op(
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
                tracing::info!(
                    credit_facility_id = %self.config.credit_facility_id,
                    "All interest accrual cycles completed for {}",
                    self.config.credit_facility_id
                );
            }
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.facility.complete_interest_cycle_and_maybe_start_new_cycle_in_op",
        skip(self, db)
        fields(credit_facility_id = %credit_facility_id),
    )]
    pub(super) async fn complete_interest_cycle_and_maybe_start_new_cycle_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
    ) -> Result<CompletedAccrualCycle, CreditFacilityError> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let (accrual_cycle_data, new_obligation) = credit_facility
            .record_interest_accrual_cycle()?
            .expect("record_interest_accrual_cycle should execute when there is an accrual cycle to record");

        if let Some(new_obligation) = new_obligation {
            self.collections
                .obligations()
                .create_with_jobs_in_op(db, new_obligation)
                .await?;
        };

        let res = credit_facility
            .start_interest_accrual_cycle()?
            .expect("start_interest_accrual_cycle always returns Executed");
        self.credit_facility_repo
            .update_in_op(db, &mut credit_facility)
            .await?;

        let new_cycle_data = res.map(|periods| {
            let new_accrual_cycle_id = credit_facility
                .interest_accrual_cycle_in_progress()
                .expect("First accrual cycle not found")
                .id;

            NewInterestAccrualCycleData {
                id: new_accrual_cycle_id,
                first_accrual_end_date: periods.accrual.end,
            }
        });

        Ok(CompletedAccrualCycle {
            facility_accrual_cycle_data: (accrual_cycle_data, credit_facility.account_ids).into(),
            new_cycle_data,
        })
    }
}

pub type InterestAccrualJobSpawner<Perms, E> = JobSpawner<InterestAccrualJobConfig<Perms, E>>;
