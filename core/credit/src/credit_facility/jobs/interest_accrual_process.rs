//! Interest Accrual Process Manager
//!
//! A per-facility process manager that coordinates the interest accrual
//! lifecycle by spawning command jobs and listening for their completion
//! events via the outbox.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                   InterestAccrualProcessState                      │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  SpawningAccrual                                                   │
//! │    • Capture outbox sequence                                       │
//! │    • Spawn AccrueInterestCommand                                   │
//! │    → transition to AwaitingAccrual                                 │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  AwaitingAccrual { start_sequence }                                │
//! │    • Listen for CoreCreditEvent::InterestAccrued                   │
//! │    → success: transition to SpawningCycleCompletion                │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  SpawningCycleCompletion                                           │
//! │    • Capture outbox sequence                                       │
//! │    • Spawn CompleteAccrualCycleCommand                             │
//! │    → transition to AwaitingCycleCompletion                         │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  AwaitingCycleCompletion { start_sequence }                        │
//! │    • Listen for CoreCreditEvent::AccrualPosted                     │
//! │    → success: transition to Completed                              │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  Completed                                                         │
//! │    • Complete the job                                              │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use job::*;
use obix::out::{Outbox, OutboxEventMarker};

use core_time_events::interest_accrual_process::INTEREST_ACCRUAL_PROCESS_JOB_TYPE;

use super::accrue_interest_command::{AccrueInterestCommandConfig, AccrueInterestCommandSpawner};
use super::complete_accrual_cycle_command::{
    CompleteAccrualCycleCommandConfig, CompleteAccrualCycleCommandSpawner,
};
use crate::{
    CreditFacilityId,
    public::{CoreCreditEvent, PublicInterestAccrualCycle},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestAccrualProcessConfig {
    pub credit_facility_id: CreditFacilityId,
    pub date: NaiveDate,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
enum InterestAccrualProcessState {
    /// Capture outbox sequence and spawn AccrueInterestCommand.
    #[default]
    SpawningAccrual,
    /// Listen for CoreCreditEvent::InterestAccrued matching this facility.
    AwaitingAccrual { start_sequence: i64 },
    /// Capture outbox sequence and spawn CompleteAccrualCycleCommand.
    SpawningCycleCompletion,
    /// Listen for CoreCreditEvent::AccrualPosted matching this facility.
    AwaitingCycleCompletion { start_sequence: i64 },
    /// Both steps completed successfully.
    Completed,
}

pub struct InterestAccrualProcessInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    accrue_spawner: AccrueInterestCommandSpawner,
    complete_spawner: CompleteAccrualCycleCommandSpawner,
}

impl<E> InterestAccrualProcessInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        accrue_spawner: AccrueInterestCommandSpawner,
        complete_spawner: CompleteAccrualCycleCommandSpawner,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            accrue_spawner,
            complete_spawner,
        }
    }
}

impl<E> JobInitializer for InterestAccrualProcessInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Config = InterestAccrualProcessConfig;

    fn job_type(&self) -> JobType {
        INTEREST_ACCRUAL_PROCESS_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(InterestAccrualProcessRunner {
            config: job.config()?,
            outbox: self.outbox.clone(),
            accrue_spawner: self.accrue_spawner.clone(),
            complete_spawner: self.complete_spawner.clone(),
        }))
    }
}

struct InterestAccrualProcessRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: InterestAccrualProcessConfig,
    outbox: Outbox<E>,
    accrue_spawner: AccrueInterestCommandSpawner,
    complete_spawner: CompleteAccrualCycleCommandSpawner,
}

impl<E> InterestAccrualProcessRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn extract_interest_accrued(event: &CoreCreditEvent) -> Option<CreditFacilityId> {
        match event {
            CoreCreditEvent::InterestAccrued {
                entity:
                    PublicInterestAccrualCycle {
                        credit_facility_id, ..
                    },
            } => Some(*credit_facility_id),
            _ => None,
        }
    }

    fn extract_accrual_posted(event: &CoreCreditEvent) -> Option<CreditFacilityId> {
        match event {
            CoreCreditEvent::AccrualPosted {
                entity:
                    PublicInterestAccrualCycle {
                        credit_facility_id, ..
                    },
            } => Some(*credit_facility_id),
            _ => None,
        }
    }

    /// Capture outbox sequence, spawn AccrueInterestCommand, transition to AwaitingAccrual.
    async fn spawn_accrual(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        process_manager::capture_and_spawn(
            &mut current_job,
            &self.outbox,
            &self.accrue_spawner,
            vec![
                JobSpec::new(
                    JobId::new(),
                    AccrueInterestCommandConfig {
                        credit_facility_id: self.config.credit_facility_id,
                    },
                )
                .queue_id(self.config.credit_facility_id.to_string()),
            ],
            |seq| InterestAccrualProcessState::AwaitingAccrual {
                start_sequence: seq,
            },
        )
        .await
    }

    /// Stream outbox events, waiting for InterestAccrued matching this facility.
    async fn await_accrual(
        &self,
        mut current_job: CurrentJob,
        start_sequence: i64,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        tracing::info!(
            start_sequence,
            credit_facility_id = %self.config.credit_facility_id,
            "Streaming outbox events for InterestAccrued"
        );

        let facility_id = self.config.credit_facility_id;
        let result = process_manager::await_event(
            &mut current_job,
            &self.outbox,
            obix::EventSequence::from(start_sequence as u64),
            |event: &CoreCreditEvent| Self::extract_interest_accrued(event) == Some(facility_id),
        )
        .await?;

        match result {
            Some(_seq) => {
                let new_state = InterestAccrualProcessState::SpawningCycleCompletion;
                let mut op = current_job.begin_op().await?;
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            None => {
                let state = InterestAccrualProcessState::AwaitingAccrual { start_sequence };
                current_job.update_execution_state(&state).await?;
                tracing::info!("Shutdown requested, rescheduling interest accrual tracking");
                Ok(JobCompletion::RescheduleIn(std::time::Duration::ZERO))
            }
        }
    }

    /// Capture outbox sequence, spawn CompleteAccrualCycleCommand, transition to AwaitingCycleCompletion.
    async fn spawn_cycle_completion(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        process_manager::capture_and_spawn(
            &mut current_job,
            &self.outbox,
            &self.complete_spawner,
            vec![
                JobSpec::new(
                    JobId::new(),
                    CompleteAccrualCycleCommandConfig {
                        credit_facility_id: self.config.credit_facility_id,
                    },
                )
                .queue_id(self.config.credit_facility_id.to_string()),
            ],
            |seq| InterestAccrualProcessState::AwaitingCycleCompletion {
                start_sequence: seq,
            },
        )
        .await
    }

    /// Stream outbox events, waiting for AccrualPosted matching this facility.
    async fn await_cycle_completion(
        &self,
        mut current_job: CurrentJob,
        start_sequence: i64,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        tracing::info!(
            start_sequence,
            credit_facility_id = %self.config.credit_facility_id,
            "Streaming outbox events for AccrualPosted"
        );

        let facility_id = self.config.credit_facility_id;
        let result = process_manager::await_event(
            &mut current_job,
            &self.outbox,
            obix::EventSequence::from(start_sequence as u64),
            |event: &CoreCreditEvent| Self::extract_accrual_posted(event) == Some(facility_id),
        )
        .await?;

        match result {
            Some(_seq) => {
                let new_state = InterestAccrualProcessState::Completed;
                let mut op = current_job.begin_op().await?;
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            None => {
                let state = InterestAccrualProcessState::AwaitingCycleCompletion { start_sequence };
                current_job.update_execution_state(&state).await?;
                tracing::info!("Shutdown requested, rescheduling cycle completion tracking");
                Ok(JobCompletion::RescheduleIn(std::time::Duration::ZERO))
            }
        }
    }
}

#[async_trait]
impl<E> JobRunner for InterestAccrualProcessRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "eod.interest-accrual-process.run",
        skip(self, current_job),
        fields(
            credit_facility_id = %self.config.credit_facility_id,
            date = %self.config.date,
        )
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<InterestAccrualProcessState>()?
            .unwrap_or_default();

        match state {
            InterestAccrualProcessState::SpawningAccrual => self.spawn_accrual(current_job).await,
            InterestAccrualProcessState::AwaitingAccrual { start_sequence } => {
                self.await_accrual(current_job, start_sequence).await
            }
            InterestAccrualProcessState::SpawningCycleCompletion => {
                self.spawn_cycle_completion(current_job).await
            }
            InterestAccrualProcessState::AwaitingCycleCompletion { start_sequence } => {
                self.await_cycle_completion(current_job, start_sequence)
                    .await
            }
            InterestAccrualProcessState::Completed => Ok(JobCompletion::Complete),
        }
    }
}

pub type InterestAccrualProcessSpawner = JobSpawner<InterestAccrualProcessConfig>;
