use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use job::*;

pub const CREDIT_FACILITY_EOD_JOB_TYPE: JobType = JobType::new("task.eod.credit-facility-eod");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditFacilityEodConfig {
    pub date: NaiveDate,
}

pub struct CreditFacilityEodJobInit {
    jobs: Jobs,
}

impl CreditFacilityEodJobInit {
    pub fn new(jobs: &Jobs) -> Self {
        Self { jobs: jobs.clone() }
    }
}

impl JobInitializer for CreditFacilityEodJobInit {
    type Config = CreditFacilityEodConfig;

    fn job_type(&self) -> JobType {
        CREDIT_FACILITY_EOD_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityEodJobRunner {
            config: job.config()?,
            jobs: self.jobs.clone(),
        }))
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CreditFacilityEodState {
    #[default]
    Collecting,
    Tracking {
        accrual_jobs: Vec<JobId>,
        maturity_jobs: Vec<JobId>,
        completed_count: usize,
        total_count: usize,
    },
    Completed,
}

struct CreditFacilityEodJobRunner {
    config: CreditFacilityEodConfig,
    jobs: Jobs,
}

#[async_trait]
impl JobRunner for CreditFacilityEodJobRunner {
    #[instrument(
        name = "eod.credit-facility-eod.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<CreditFacilityEodState>()?
            .unwrap_or_default();

        match state {
            CreditFacilityEodState::Collecting => {
                // TODO: Query credit facilities needing interest accrual for this date.
                // For each facility, spawn a per-entity accrual job with deterministic ID:
                //   let accrual_job_id = job_id::eod_entity_id(&self.config.date, "interest-accrual", &facility_id);
                //
                // TODO: Query credit facilities reaching maturity on this date.
                // For each facility, spawn a per-entity maturity job with deterministic ID:
                //   let maturity_job_id = job_id::eod_entity_id(&self.config.date, "credit-maturity", &facility_id);
                //
                // Transition to Tracking with lists of accrual_jobs and maturity_jobs.

                let new_state = CreditFacilityEodState::Completed;
                let mut op = current_job.begin_op().await?;
                current_job
                    .update_execution_state_in_op(&mut op, &new_state)
                    .await?;
                Ok(JobCompletion::RescheduleNowWithOp(op))
            }
            CreditFacilityEodState::Tracking {
                accrual_jobs,
                maturity_jobs,
                total_count,
                ..
            } => {
                let mut completed_count = 0;
                for job_id in accrual_jobs.iter().chain(maturity_jobs.iter()) {
                    match self.jobs.find(*job_id).await {
                        Ok(job) if job.completed() => completed_count += 1,
                        _ => {}
                    }
                }

                if completed_count >= total_count {
                    let new_state = CreditFacilityEodState::Completed;
                    let mut op = current_job.begin_op().await?;
                    current_job
                        .update_execution_state_in_op(&mut op, &new_state)
                        .await?;
                    Ok(JobCompletion::RescheduleNowWithOp(op))
                } else {
                    let new_state = CreditFacilityEodState::Tracking {
                        accrual_jobs,
                        maturity_jobs,
                        completed_count,
                        total_count,
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
            CreditFacilityEodState::Completed => Ok(JobCompletion::Complete),
        }
    }
}

pub type CreditFacilityEodJobSpawner = JobSpawner<CreditFacilityEodConfig>;
