use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use job::*;

pub const OBLIGATION_TRANSITION_JOB_TYPE: JobType = JobType::new("task.eod.obligation-transition");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObligationTransitionConfig {
    pub date: NaiveDate,
}

pub struct ObligationTransitionJobInit {
    jobs: Jobs,
}

impl ObligationTransitionJobInit {
    pub fn new(jobs: &Jobs) -> Self {
        Self { jobs: jobs.clone() }
    }
}

impl JobInitializer for ObligationTransitionJobInit {
    type Config = ObligationTransitionConfig;

    fn job_type(&self) -> JobType {
        OBLIGATION_TRANSITION_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationTransitionJobRunner {
            config: job.config()?,
            jobs: self.jobs.clone(),
        }))
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ObligationTransitionState {
    #[default]
    Collecting,
    Tracking {
        total: usize,
        completed: usize,
        entity_jobs: Vec<JobId>,
    },
    Completed,
}

struct ObligationTransitionJobRunner {
    config: ObligationTransitionConfig,
    jobs: Jobs,
}

#[async_trait]
impl JobRunner for ObligationTransitionJobRunner {
    #[instrument(
        name = "eod.obligation-transition.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<ObligationTransitionState>()?
            .unwrap_or_default();

        match state {
            ObligationTransitionState::Collecting => {
                // TODO: Query obligations needing transition for this date.
                // For each obligation, spawn a per-entity job with deterministic ID:
                //   let entity_job_id = job_id::eod_entity_id(&self.config.date, "obligation-transition", &obligation_id);
                // Transition to Tracking with list of entity_jobs.

                let new_state = ObligationTransitionState::Completed;
                let mut op = current_job.begin_op().await?;
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            ObligationTransitionState::Tracking {
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
                    let new_state = ObligationTransitionState::Completed;
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    let new_state = ObligationTransitionState::Tracking {
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
            ObligationTransitionState::Completed => Ok(JobCompletion::Complete),
        }
    }
}

pub type ObligationTransitionJobSpawner = JobSpawner<ObligationTransitionConfig>;
