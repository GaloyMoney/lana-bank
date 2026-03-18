use async_trait::async_trait;
use chrono::NaiveDate;
use job::{
    CurrentJob, Job, JobCompletion, JobInitializer, JobRunner, JobSpawner, JobSpec, JobType,
    RetrySettings,
};
use serde::{Deserialize, Serialize};

use domain_config::ExposedDomainConfigsReadOnly;

use crate::{
    ClosingSchedule,
    config::{ClosingTime, Timezone},
    job_id,
    process_manager::{EodProcessManagerConfig, EodProcessManagerJobSpawner},
};

#[derive(Default, Clone, Serialize, Deserialize)]
struct EndOfDayProducerState {
    last_published_day: Option<NaiveDate>,
}

#[derive(Deserialize, Serialize)]
pub struct EndOfDayProducerJobConfig {}

pub struct EndOfDayProducerJobInit {
    pm_spawner: EodProcessManagerJobSpawner,
    domain_configs: ExposedDomainConfigsReadOnly,
}

impl EndOfDayProducerJobInit {
    pub fn new(
        pm_spawner: &EodProcessManagerJobSpawner,
        domain_configs: &ExposedDomainConfigsReadOnly,
    ) -> Self {
        Self {
            pm_spawner: pm_spawner.clone(),
            domain_configs: domain_configs.clone(),
        }
    }
}

pub const END_OF_DAY_PRODUCER_JOB: JobType =
    JobType::new("cron.core-time-event.end-of-day-producer");

impl JobInitializer for EndOfDayProducerJobInit {
    type Config = EndOfDayProducerJobConfig;

    fn job_type(&self) -> JobType {
        END_OF_DAY_PRODUCER_JOB
    }

    fn init(
        &self,
        _: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EndOfDayProducerJobRunner {
            pm_spawner: self.pm_spawner.clone(),
            domain_configs: self.domain_configs.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct EndOfDayProducerJobRunner {
    pm_spawner: EodProcessManagerJobSpawner,
    domain_configs: ExposedDomainConfigsReadOnly,
}

#[async_trait]
impl JobRunner for EndOfDayProducerJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<EndOfDayProducerState>()?
            .unwrap_or_default();

        let closing_time_config = self
            .domain_configs
            .get_without_audit::<ClosingTime>()
            .await?;
        let timezone_config = self.domain_configs.get_without_audit::<Timezone>().await?;
        let closing_time = closing_time_config.value();
        let timezone = timezone_config.value();
        let clock = current_job.clock().clone();

        let schedule = ClosingSchedule::new(timezone, closing_time, &clock);
        let most_recent_closing_day = schedule.next_closing_day() - chrono::Days::new(1);

        match state.last_published_day {
            None => {
                state.last_published_day = Some(most_recent_closing_day);
                current_job.update_execution_state(&state).await?;

                tracing::info!(
                    job_id = %current_job.id(),
                    job_type = %END_OF_DAY_PRODUCER_JOB,
                    day = %most_recent_closing_day,
                    "Initialized last_published_day"
                );
            }
            Some(last_day) if last_day < most_recent_closing_day => {
                let mut day = last_day + chrono::Days::new(1);
                while day <= most_recent_closing_day {
                    let closing_dt = ClosingSchedule::closing_for_day(timezone, closing_time, day);

                    let process_id = job_id::eod_process_id_from_date(&day);
                    let manager_job_id = job_id::eod_manager_id(&day);

                    let mut op = current_job.begin_op().await?;
                    let spec = JobSpec::new(
                        manager_job_id,
                        EodProcessManagerConfig {
                            date: day,
                            closing_time: closing_dt,
                            process_id,
                        },
                    )
                    .queue_id("eod-manager".to_string());
                    match self.pm_spawner.spawn_all_in_op(&mut op, vec![spec]).await {
                        Ok(_) | Err(job::error::JobError::DuplicateId(_)) => {}
                        Err(e) => return Err(e.into()),
                    }
                    state.last_published_day = Some(day);
                    current_job
                        .update_execution_state_in_op(&mut op, &state)
                        .await?;
                    op.commit().await?;

                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %END_OF_DAY_PRODUCER_JOB,
                        day = %day,
                        closing_time = %closing_dt,
                        "EOD process manager job spawned"
                    );

                    day = day + chrono::Days::new(1);
                }
            }
            _ => {
                tracing::debug!(
                    job_id = %current_job.id(),
                    job_type = %END_OF_DAY_PRODUCER_JOB,
                    "Already caught up"
                );
            }
        }

        Ok(JobCompletion::RescheduleAt(schedule.next_closing()))
    }
}
