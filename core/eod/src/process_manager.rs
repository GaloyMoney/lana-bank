use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDate;
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

/// Polling interval when checking child job completion.
const POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EodProcessManagerConfig {
    pub date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EodProcessState {
    #[default]
    SpawningPhase1,
    AwaitingPhase1 {
        obligation_job: JobId,
        deposit_job: JobId,
    },
    SpawningPhase2,
    AwaitingPhase2 {
        credit_facility_job: JobId,
    },
    Completed,
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
                Ok(JobCompletion::RescheduleInWithOp(op, POLL_INTERVAL))
            }
            EodProcessState::AwaitingPhase1 {
                obligation_job,
                deposit_job,
            } => {
                let obligation_completed = match self.jobs.find(obligation_job).await {
                    Ok(job) => job.completed(),
                    Err(e) => {
                        tracing::warn!(
                            job_id = %obligation_job,
                            error = %e,
                            "Could not find obligation child job, will retry"
                        );
                        false
                    }
                };
                let deposit_completed = match self.jobs.find(deposit_job).await {
                    Ok(job) => job.completed(),
                    Err(e) => {
                        tracing::warn!(
                            job_id = %deposit_job,
                            error = %e,
                            "Could not find deposit child job, will retry"
                        );
                        false
                    }
                };

                if obligation_completed && deposit_completed {
                    let new_state = EodProcessState::SpawningPhase2;
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    Ok(JobCompletion::RescheduleIn(POLL_INTERVAL))
                }
            }
            EodProcessState::SpawningPhase2 => {
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
                    credit_facility_job,
                };
                let mut op = current_job.begin_op().await?;
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleInWithOp(op, POLL_INTERVAL))
            }
            EodProcessState::AwaitingPhase2 {
                credit_facility_job,
            } => {
                let completed = match self.jobs.find(credit_facility_job).await {
                    Ok(job) => job.completed(),
                    Err(e) => {
                        tracing::warn!(
                            job_id = %credit_facility_job,
                            error = %e,
                            "Could not find credit facility child job, will retry"
                        );
                        false
                    }
                };

                if completed {
                    let new_state = EodProcessState::Completed;
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    Ok(JobCompletion::RescheduleIn(POLL_INTERVAL))
                }
            }
            EodProcessState::Completed => Ok(JobCompletion::Complete),
        }
    }
}

pub type EodProcessManagerJobSpawner = JobSpawner<EodProcessManagerConfig>;
