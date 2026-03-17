use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use job::*;

pub const DEPOSIT_ACTIVITY_JOB_TYPE: JobType = JobType::new("task.eod.deposit-activity");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositActivityConfig {
    pub date: NaiveDate,
}

pub struct DepositActivityJobInit {
    jobs: Jobs,
}

impl DepositActivityJobInit {
    pub fn new(jobs: &Jobs) -> Self {
        Self { jobs: jobs.clone() }
    }
}

impl JobInitializer for DepositActivityJobInit {
    type Config = DepositActivityConfig;

    fn job_type(&self) -> JobType {
        DEPOSIT_ACTIVITY_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DepositActivityJobRunner {
            config: job.config()?,
            jobs: self.jobs.clone(),
        }))
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum DepositActivityState {
    #[default]
    Collecting,
    Tracking {
        total: usize,
        completed: usize,
        entity_jobs: Vec<JobId>,
    },
    Completed,
}

struct DepositActivityJobRunner {
    config: DepositActivityConfig,
    jobs: Jobs,
}

#[async_trait]
impl JobRunner for DepositActivityJobRunner {
    #[instrument(
        name = "eod.deposit-activity.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<DepositActivityState>()?
            .unwrap_or_default();

        match state {
            DepositActivityState::Collecting => {
                // TODO: Query deposit accounts needing activity evaluation for this date.
                // For each account, spawn a per-entity job with deterministic ID:
                //   let entity_job_id = job_id::eod_entity_id(&self.config.date, "deposit-activity", &account_id);
                // Transition to Tracking with list of entity_jobs.

                let new_state = DepositActivityState::Completed;
                let mut op = current_job.begin_op().await?;
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            DepositActivityState::Tracking {
                entity_jobs, total, ..
            } => {
                let mut completed_count = 0;
                for job_id in &entity_jobs {
                    match self.jobs.find(*job_id).await {
                        Ok(job) if job.completed() => completed_count += 1,
                        _ => {}
                    }
                }

                if completed_count >= total {
                    let new_state = DepositActivityState::Completed;
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    let new_state = DepositActivityState::Tracking {
                        total,
                        completed: completed_count,
                        entity_jobs,
                    };
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleInWithOp(
                        op,
                        std::time::Duration::from_secs(5),
                    ))
                }
            }
            DepositActivityState::Completed => Ok(JobCompletion::Complete),
        }
    }
}

pub type DepositActivityJobSpawner = JobSpawner<DepositActivityConfig>;
