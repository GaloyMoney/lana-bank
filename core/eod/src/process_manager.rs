use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use job::*;

use crate::job_id;

pub const EOD_PROCESS_MANAGER_JOB_TYPE: JobType = JobType::new("task.eod.process-manager");

/// Polling interval when checking execution state for child completion.
const POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EodProcessManagerConfig {
    pub date: NaiveDate,
}

/// Status of a child sub-process, tracked inline in the PM's execution state.
///
/// Updated externally by outbox event handlers when children complete or fail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChildJobStatus {
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EodProcessState {
    #[default]
    SpawningPhase1,
    AwaitingPhase1 {
        obligation_job: JobId,
        obligation_status: ChildJobStatus,
        deposit_job: JobId,
        deposit_status: ChildJobStatus,
    },
    SpawningPhase2,
    AwaitingPhase2 {
        credit_facility_job: JobId,
        credit_facility_status: ChildJobStatus,
    },
    Completed,
    Failed {
        phase: u8,
        reason: String,
    },
}

#[derive(Default)]
pub struct EodProcessManagerJobInit {}

impl EodProcessManagerJobInit {
    pub fn new() -> Self {
        Self {}
    }
}

impl JobInitializer for EodProcessManagerJobInit {
    type Config = EodProcessManagerConfig;

    fn job_type(&self) -> JobType {
        EOD_PROCESS_MANAGER_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EodProcessManagerJobRunner {
            config: job.config()?,
        }))
    }
}

struct EodProcessManagerJobRunner {
    config: EodProcessManagerConfig,
}

#[async_trait]
impl JobRunner for EodProcessManagerJobRunner {
    #[instrument(
        name = "eod.process-manager.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<EodProcessState>()?
            .unwrap_or_default();

        match state {
            EodProcessState::SpawningPhase1 => {
                let obligation_job =
                    job_id::eod_child_id(&self.config.date, "obligation-transition");
                let deposit_job = job_id::eod_child_id(&self.config.date, "deposit-activity");

                // TODO(Phase 2): spawn obligation-transition child job
                // match self.obligation_spawner.spawn(obligation_job, config).await {
                //     Ok(_) | Err(JobError::DuplicateId(_)) => {}
                //     Err(e) => return Err(e.into()),
                // }

                // TODO(Phase 2): spawn deposit-activity child job
                // match self.deposit_spawner.spawn(deposit_job, config).await {
                //     Ok(_) | Err(JobError::DuplicateId(_)) => {}
                //     Err(e) => return Err(e.into()),
                // }

                let new_state = EodProcessState::AwaitingPhase1 {
                    obligation_job,
                    obligation_status: ChildJobStatus::Running,
                    deposit_job,
                    deposit_status: ChildJobStatus::Running,
                };
                let mut op = current_job.begin_op().await?;
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleInWithOp(op, POLL_INTERVAL))
            }
            EodProcessState::AwaitingPhase1 {
                obligation_status,
                deposit_status,
                ..
            } => {
                // Statuses are updated by outbox event handlers.
                // Check whether all Phase 1 children have reached a terminal state.
                match (&obligation_status, &deposit_status) {
                    (ChildJobStatus::Completed, ChildJobStatus::Completed) => {
                        let new_state = EodProcessState::SpawningPhase2;
                        let mut op = current_job.begin_op().await?;
                        current_job
                            .update_execution_state_in_op(&mut op, &new_state)
                            .await?;
                        Ok(JobCompletion::RescheduleNowWithOp(op))
                    }
                    (ChildJobStatus::Failed(reason), _) | (_, ChildJobStatus::Failed(reason)) => {
                        let new_state = EodProcessState::Failed {
                            phase: 1,
                            reason: reason.clone(),
                        };
                        let mut op = current_job.begin_op().await?;
                        current_job
                            .update_execution_state_in_op(&mut op, &new_state)
                            .await?;
                        Ok(JobCompletion::RescheduleNowWithOp(op))
                    }
                    _ => {
                        // At least one child still running — poll again
                        Ok(JobCompletion::RescheduleIn(POLL_INTERVAL))
                    }
                }
            }
            EodProcessState::SpawningPhase2 => {
                let credit_facility_job =
                    job_id::eod_child_id(&self.config.date, "credit-facility");

                // TODO(Phase 2): spawn credit-facility-eod child job
                // match self.credit_spawner.spawn(credit_facility_job, config).await {
                //     Ok(_) | Err(JobError::DuplicateId(_)) => {}
                //     Err(e) => return Err(e.into()),
                // }

                let new_state = EodProcessState::AwaitingPhase2 {
                    credit_facility_job,
                    credit_facility_status: ChildJobStatus::Running,
                };
                let mut op = current_job.begin_op().await?;
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleInWithOp(op, POLL_INTERVAL))
            }
            EodProcessState::AwaitingPhase2 {
                credit_facility_status,
                ..
            } => match &credit_facility_status {
                ChildJobStatus::Completed => {
                    let new_state = EodProcessState::Completed;
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                }
                ChildJobStatus::Failed(reason) => {
                    let new_state = EodProcessState::Failed {
                        phase: 2,
                        reason: reason.clone(),
                    };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                }
                ChildJobStatus::Running => Ok(JobCompletion::RescheduleIn(POLL_INTERVAL)),
            },
            EodProcessState::Completed | EodProcessState::Failed { .. } => {
                Ok(JobCompletion::Complete)
            }
        }
    }
}

pub type EodProcessManagerJobSpawner = JobSpawner<EodProcessManagerConfig>;
