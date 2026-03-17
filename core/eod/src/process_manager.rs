use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use job::{error::JobError, *};

use crate::{
    credit_facility_eod::{CreditFacilityEodConfig, CreditFacilityEodJobSpawner},
    deposit_activity::{DepositActivityConfig, DepositActivityJobSpawner},
    job_id,
    obligation_transition::{ObligationTransitionConfig, ObligationTransitionJobSpawner},
};

pub const EOD_PROCESS_MANAGER_JOB_TYPE: JobType = JobType::new("task.eod.process-manager");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EodProcessManagerConfig {
    pub date: NaiveDate,
    pub closing_time: DateTime<Utc>,
}

/// Structured result set on the process manager job upon completion.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EodProcessManagerResult {
    pub date: NaiveDate,
    pub phase1_obligation: JobTerminalState,
    pub phase1_deposit: JobTerminalState,
    pub phase2_credit_facility: JobTerminalState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EodProcessState {
    #[default]
    SpawningPhase1,
    AwaitingPhase1 {
        obligation_job: JobId,
        deposit_job: JobId,
    },
    SpawningPhase2 {
        phase1_obligation: JobTerminalState,
        phase1_deposit: JobTerminalState,
    },
    AwaitingPhase2 {
        phase1_obligation: JobTerminalState,
        phase1_deposit: JobTerminalState,
        credit_facility_job: JobId,
    },
    Cancelling {
        phase: u8,
    },
    Completed,
    Failed {
        phase: u8,
        failed: Vec<(String, JobTerminalState)>,
    },
}

pub struct EodProcessManagerJobInit {
    jobs: Jobs,
    obligation_spawner: ObligationTransitionJobSpawner,
    deposit_spawner: DepositActivityJobSpawner,
    credit_facility_spawner: CreditFacilityEodJobSpawner,
}

impl EodProcessManagerJobInit {
    pub fn new(
        jobs: &Jobs,
        obligation_spawner: ObligationTransitionJobSpawner,
        deposit_spawner: DepositActivityJobSpawner,
        credit_facility_spawner: CreditFacilityEodJobSpawner,
    ) -> Self {
        Self {
            jobs: jobs.clone(),
            obligation_spawner,
            deposit_spawner,
            credit_facility_spawner,
        }
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
            jobs: self.jobs.clone(),
            obligation_spawner: self.obligation_spawner.clone(),
            deposit_spawner: self.deposit_spawner.clone(),
            credit_facility_spawner: self.credit_facility_spawner.clone(),
        }))
    }
}

struct EodProcessManagerJobRunner {
    config: EodProcessManagerConfig,
    jobs: Jobs,
    obligation_spawner: ObligationTransitionJobSpawner,
    deposit_spawner: DepositActivityJobSpawner,
    credit_facility_spawner: CreditFacilityEodJobSpawner,
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

                match self
                    .obligation_spawner
                    .spawn(
                        obligation_job,
                        ObligationTransitionConfig {
                            date: self.config.date,
                        },
                    )
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                match self
                    .deposit_spawner
                    .spawn(
                        deposit_job,
                        DepositActivityConfig {
                            date: self.config.date,
                            closing_time: self.config.closing_time,
                        },
                    )
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                let new_state = EodProcessState::AwaitingPhase1 {
                    obligation_job,
                    deposit_job,
                };
                let mut op = current_job.begin_op().await?;
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            EodProcessState::AwaitingPhase1 {
                obligation_job,
                deposit_job,
            } => {
                if current_job.cancellation_requested() {
                    tracing::info!("EOD process manager cancellation requested during phase 1");
                    let new_state = EodProcessState::Cancelling { phase: 1 };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    return Ok(JobCompletion::RescheduleNowWithOp(op));
                }

                let (obligation_result, deposit_result) = tokio::join!(
                    self.jobs.await_completion(obligation_job),
                    self.jobs.await_completion(deposit_job),
                );
                let obligation_terminal = obligation_result?;
                let deposit_terminal = deposit_result?;

                let mut failed = Vec::new();
                if obligation_terminal != JobTerminalState::Completed {
                    failed.push(("obligation-transition".to_string(), obligation_terminal));
                }
                if deposit_terminal != JobTerminalState::Completed {
                    failed.push(("deposit-activity".to_string(), deposit_terminal));
                }

                if !failed.is_empty() {
                    let new_state = EodProcessState::Failed { phase: 1, failed };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    let new_state = EodProcessState::SpawningPhase2 {
                        phase1_obligation: obligation_terminal,
                        phase1_deposit: deposit_terminal,
                    };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                }
            }
            EodProcessState::SpawningPhase2 {
                phase1_obligation,
                phase1_deposit,
            } => {
                let credit_facility_job =
                    job_id::eod_child_id(&self.config.date, "credit-facility");

                match self
                    .credit_facility_spawner
                    .spawn(
                        credit_facility_job,
                        CreditFacilityEodConfig {
                            date: self.config.date,
                        },
                    )
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                let new_state = EodProcessState::AwaitingPhase2 {
                    phase1_obligation,
                    phase1_deposit,
                    credit_facility_job,
                };
                let mut op = current_job.begin_op().await?;
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            EodProcessState::AwaitingPhase2 {
                phase1_obligation,
                phase1_deposit,
                credit_facility_job,
            } => {
                if current_job.cancellation_requested() {
                    tracing::info!("EOD process manager cancellation requested during phase 2");
                    let new_state = EodProcessState::Cancelling { phase: 2 };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    return Ok(JobCompletion::RescheduleNowWithOp(op));
                }

                let credit_facility_terminal =
                    self.jobs.await_completion(credit_facility_job).await?;

                if credit_facility_terminal != JobTerminalState::Completed {
                    let new_state = EodProcessState::Failed {
                        phase: 2,
                        failed: vec![("credit-facility".to_string(), credit_facility_terminal)],
                    };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    let result = EodProcessManagerResult {
                        date: self.config.date,
                        phase1_obligation,
                        phase1_deposit,
                        phase2_credit_facility: credit_facility_terminal,
                    };
                    current_job.set_result(&result).await?;

                    let new_state = EodProcessState::Completed;
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                }
            }
            EodProcessState::Cancelling { phase } => {
                // TODO: Once the job library exposes a cancel API, use it here
                // to cancel running child jobs and await their acknowledgment.
                tracing::warn!(
                    phase,
                    "EOD process manager entering cancelling state — \
                     child job cancellation not yet implemented"
                );
                Ok(JobCompletion::Complete)
            }
            EodProcessState::Completed => Ok(JobCompletion::Complete),
            EodProcessState::Failed { phase, ref failed } => {
                tracing::error!(
                    phase,
                    ?failed,
                    "EOD process manager failed — manual intervention required"
                );
                Ok(JobCompletion::Complete)
            }
        }
    }
}

pub type EodProcessManagerJobSpawner = JobSpawner<EodProcessManagerConfig>;
