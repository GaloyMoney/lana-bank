//! Interest Accrual Process Manager
//!
//! A per-facility process manager that coordinates the interest accrual
//! lifecycle by spawning and awaiting two command jobs:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                   InterestAccrualProcessState                      │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  AccruingInterest                                                  │
//! │    • Spawn AccrueInterestCommand                                   │
//! │    • Await completion                                              │
//! │    → success: transition to CompletingCycle                        │
//! │    → failure: transition to Failed                                 │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  CompletingCycle                                                   │
//! │    • Spawn CompleteAccrualCycleCommand                             │
//! │    • Await completion                                              │
//! │    → success: transition to Completed                              │
//! │    → failure: transition to Failed                                 │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  Completed                                                         │
//! │    • Set result and complete                                       │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  Failed                                                            │
//! │    • Record failure and complete                                   │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use job::{error::JobError, *};

use core_eod::interest_accrual_process::INTEREST_ACCRUAL_PROCESS_JOB_TYPE;

use super::accrue_interest_command::{AccrueInterestCommandConfig, AccrueInterestCommandSpawner};
use super::complete_accrual_cycle_command::{
    CompleteAccrualCycleCommandConfig, CompleteAccrualCycleCommandSpawner,
};
use crate::CreditFacilityId;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestAccrualProcessConfig {
    pub credit_facility_id: CreditFacilityId,
    pub date: NaiveDate,
}

/// Result set on the process manager job upon completion.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestAccrualProcessResult {
    pub credit_facility_id: CreditFacilityId,
    pub accrual_terminal: JobTerminalState,
    pub cycle_terminal: Option<JobTerminalState>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
enum InterestAccrualProcessState {
    /// Spawn AccrueInterestCommand and await its completion.
    #[default]
    AccruingInterest,
    /// AccrueInterestCommand spawned; awaiting completion.
    AwaitingAccrual { accrual_job: JobId },
    /// Spawn CompleteAccrualCycleCommand and await its completion.
    SpawningCycleCompletion { accrual_terminal: JobTerminalState },
    /// CompleteAccrualCycleCommand spawned; awaiting completion.
    AwaitingCycleCompletion {
        accrual_terminal: JobTerminalState,
        cycle_job: JobId,
    },
    /// Both commands completed successfully.
    Completed {
        result: InterestAccrualProcessResult,
    },
    /// One of the child commands failed.
    Failed {
        result: InterestAccrualProcessResult,
    },
}

pub struct InterestAccrualProcessInit {
    jobs: Jobs,
    accrue_spawner: AccrueInterestCommandSpawner,
    complete_spawner: CompleteAccrualCycleCommandSpawner,
}

impl InterestAccrualProcessInit {
    pub fn new(
        jobs: &Jobs,
        accrue_spawner: AccrueInterestCommandSpawner,
        complete_spawner: CompleteAccrualCycleCommandSpawner,
    ) -> Self {
        Self {
            jobs: jobs.clone(),
            accrue_spawner,
            complete_spawner,
        }
    }
}

impl JobInitializer for InterestAccrualProcessInit {
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
            jobs: self.jobs.clone(),
            accrue_spawner: self.accrue_spawner.clone(),
            complete_spawner: self.complete_spawner.clone(),
        }))
    }
}

struct InterestAccrualProcessRunner {
    config: InterestAccrualProcessConfig,
    jobs: Jobs,
    accrue_spawner: AccrueInterestCommandSpawner,
    complete_spawner: CompleteAccrualCycleCommandSpawner,
}

#[async_trait]
impl JobRunner for InterestAccrualProcessRunner {
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
            InterestAccrualProcessState::AccruingInterest => {
                let accrual_job = core_eod::eod_entity_id(
                    &self.config.date,
                    "accrue-interest",
                    &(*self.config.credit_facility_id).into(),
                );

                let mut op = current_job.begin_op().await?;

                match self
                    .accrue_spawner
                    .spawn_all_in_op(
                        &mut op,
                        vec![
                            JobSpec::new(
                                accrual_job,
                                AccrueInterestCommandConfig {
                                    credit_facility_id: self.config.credit_facility_id,
                                },
                            )
                            .queue_id(self.config.credit_facility_id.to_string()),
                        ],
                    )
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                let new_state = InterestAccrualProcessState::AwaitingAccrual { accrual_job };
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }

            InterestAccrualProcessState::AwaitingAccrual { accrual_job } => {
                let state = InterestAccrualProcessState::AwaitingAccrual { accrual_job };
                let accrual_terminal = tokio::select! {
                    result = self.jobs.await_completion(accrual_job) => result?,
                    _ = current_job.shutdown_requested() => {
                        current_job.update_execution_state(&state).await?;
                        return Ok(JobCompletion::RescheduleIn(std::time::Duration::ZERO));
                    }
                };

                if accrual_terminal != JobTerminalState::Completed {
                    tracing::error!(
                        ?accrual_terminal,
                        credit_facility_id = %self.config.credit_facility_id,
                        "AccrueInterestCommand failed"
                    );
                    let result = InterestAccrualProcessResult {
                        credit_facility_id: self.config.credit_facility_id,
                        accrual_terminal,
                        cycle_terminal: None,
                    };
                    let new_state = InterestAccrualProcessState::Failed { result };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    let new_state =
                        InterestAccrualProcessState::SpawningCycleCompletion { accrual_terminal };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                }
            }

            InterestAccrualProcessState::SpawningCycleCompletion { accrual_terminal } => {
                let cycle_job = core_eod::eod_entity_id(
                    &self.config.date,
                    "complete-accrual-cycle",
                    &(*self.config.credit_facility_id).into(),
                );

                let mut op = current_job.begin_op().await?;

                match self
                    .complete_spawner
                    .spawn_all_in_op(
                        &mut op,
                        vec![
                            JobSpec::new(
                                cycle_job,
                                CompleteAccrualCycleCommandConfig {
                                    credit_facility_id: self.config.credit_facility_id,
                                },
                            )
                            .queue_id(self.config.credit_facility_id.to_string()),
                        ],
                    )
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                let new_state = InterestAccrualProcessState::AwaitingCycleCompletion {
                    accrual_terminal,
                    cycle_job,
                };
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }

            InterestAccrualProcessState::AwaitingCycleCompletion {
                accrual_terminal,
                cycle_job,
            } => {
                let state = InterestAccrualProcessState::AwaitingCycleCompletion {
                    accrual_terminal,
                    cycle_job,
                };
                let cycle_terminal = tokio::select! {
                    result = self.jobs.await_completion(cycle_job) => result?,
                    _ = current_job.shutdown_requested() => {
                        current_job.update_execution_state(&state).await?;
                        return Ok(JobCompletion::RescheduleIn(std::time::Duration::ZERO));
                    }
                };

                let result = InterestAccrualProcessResult {
                    credit_facility_id: self.config.credit_facility_id,
                    accrual_terminal,
                    cycle_terminal: Some(cycle_terminal),
                };

                if cycle_terminal != JobTerminalState::Completed {
                    tracing::error!(
                        ?cycle_terminal,
                        credit_facility_id = %self.config.credit_facility_id,
                        "CompleteAccrualCycleCommand failed"
                    );
                    let new_state = InterestAccrualProcessState::Failed { result };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    let new_state = InterestAccrualProcessState::Completed { result };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                }
            }

            InterestAccrualProcessState::Completed { result } => {
                current_job.set_result(&result).await?;
                Ok(JobCompletion::Complete)
            }

            InterestAccrualProcessState::Failed { result } => {
                current_job.set_result(&result).await?;
                Err(format!(
                    "InterestAccrualProcess failed for facility {}: accrual={:?}, cycle={:?}",
                    result.credit_facility_id, result.accrual_terminal, result.cycle_terminal
                )
                .into())
            }
        }
    }
}

pub type InterestAccrualProcessSpawner = JobSpawner<InterestAccrualProcessConfig>;
