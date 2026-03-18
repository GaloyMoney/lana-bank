use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

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
pub(crate) enum EodProcessState {
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
    Completed {
        result: EodProcessManagerResult,
    },
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
    #[record_error_severity]
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

                let mut op = current_job.begin_op().await?;

                match self
                    .obligation_spawner
                    .spawn_all_in_op(
                        &mut op,
                        vec![JobSpec::new(
                            obligation_job,
                            ObligationTransitionConfig {
                                date: self.config.date,
                            },
                        )
                        .queue_id("eod-obligation-transition".to_string())],
                    )
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                match self
                    .deposit_spawner
                    .spawn_all_in_op(
                        &mut op,
                        vec![JobSpec::new(
                            deposit_job,
                            DepositActivityConfig {
                                date: self.config.date,
                                closing_time: self.config.closing_time,
                            },
                        )
                        .queue_id("eod-deposit-activity".to_string())],
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
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            EodProcessState::AwaitingPhase1 {
                obligation_job,
                deposit_job,
            } => {
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
                    tracing::error!(
                        phase = 1,
                        ?failed,
                        "EOD process manager failed — manual intervention required"
                    );
                    let new_state = EodProcessState::Failed { phase: 1, failed };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::CompleteWithOp(op))
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

                let mut op = current_job.begin_op().await?;

                match self
                    .credit_facility_spawner
                    .spawn_all_in_op(
                        &mut op,
                        vec![JobSpec::new(
                            credit_facility_job,
                            CreditFacilityEodConfig {
                                date: self.config.date,
                            },
                        )
                        .queue_id("eod-credit-facility".to_string())],
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
                let credit_facility_terminal =
                    self.jobs.await_completion(credit_facility_job).await?;

                if credit_facility_terminal != JobTerminalState::Completed {
                    let failed = vec![("credit-facility".to_string(), credit_facility_terminal)];
                    tracing::error!(
                        phase = 2,
                        ?failed,
                        "EOD process manager failed — manual intervention required"
                    );
                    let new_state = EodProcessState::Failed { phase: 2, failed };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::CompleteWithOp(op))
                } else {
                    let result = EodProcessManagerResult {
                        date: self.config.date,
                        phase1_obligation,
                        phase1_deposit,
                        phase2_credit_facility: credit_facility_terminal,
                    };
                    // Checkpoint to Completed first; set_result runs on the
                    // next iteration from the Completed arm, making this
                    // crash-safe: if we crash between checkpoint and
                    // set_result the PM simply retries set_result.
                    let new_state = EodProcessState::Completed { result };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                }
            }
            EodProcessState::Completed { result } => {
                // Idempotent: set_result may have already been called before
                // a crash; calling it again is safe.
                current_job.set_result(&result).await?;
                Ok(JobCompletion::Complete)
            }
            EodProcessState::Failed { .. } => {
                // Already logged when transitioning to Failed; this arm
                // only runs if the PM is restarted after a crash.
                Ok(JobCompletion::Complete)
            }
        }
    }
}

pub type EodProcessManagerJobSpawner = JobSpawner<EodProcessManagerConfig>;
