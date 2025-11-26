use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

use job::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWaitJobConfig {
    /// How long the job should run for (in seconds)
    #[serde(default = "default_wait_seconds")]
    pub wait_seconds: u64,
    /// Whether to respect shutdown signals
    #[serde(default = "default_respect_shutdown")]
    pub respect_shutdown: bool,
}

fn default_wait_seconds() -> u64 {
    30
}

fn default_respect_shutdown() -> bool {
    true
}

impl Default for TestWaitJobConfig {
    fn default() -> Self {
        Self {
            wait_seconds: default_wait_seconds(),
            respect_shutdown: default_respect_shutdown(),
        }
    }
}

impl JobConfig for TestWaitJobConfig {
    type Initializer = TestWaitJobInit;
}

pub struct TestWaitJobInit;

impl TestWaitJobInit {
    pub fn new() -> Self {
        Self
    }
}

const TEST_WAIT_JOB_TYPE: JobType = JobType::new("test.wait-job");

impl JobInitializer for TestWaitJobInit {
    fn job_type() -> JobType {
        TEST_WAIT_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let config: TestWaitJobConfig = job.config()?;
        Ok(Box::new(TestWaitJobRunner { config }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings {
            n_attempts: Some(3),
            n_warn_attempts: Some(2),
            min_backoff: Duration::from_secs(5),
            max_backoff: Duration::from_secs(60),
            backoff_jitter_pct: 10,
            attempt_reset_after_backoff_multiples: 5,
        }
    }
}

pub struct TestWaitJobRunner {
    config: TestWaitJobConfig,
}

#[async_trait]
impl JobRunner for TestWaitJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        tracing::info!(
            job_id = %current_job.id(),
            wait_seconds = self.config.wait_seconds,
            respect_shutdown = self.config.respect_shutdown,
            "Test wait job started"
        );

        let total_wait = Duration::from_secs(self.config.wait_seconds);
        let check_interval = Duration::from_millis(500);
        let mut elapsed = Duration::ZERO;

        while elapsed < total_wait {
            let remaining = total_wait - elapsed;
            let sleep_duration = if remaining < check_interval {
                remaining
            } else {
                check_interval
            };

            if self.config.respect_shutdown {
                tokio::select! {
                    _ = sleep(sleep_duration) => {
                        elapsed += sleep_duration;
                        if elapsed.as_secs() % 5 == 0 {
                            tracing::info!(
                                job_id = %current_job.id(),
                                elapsed_secs = elapsed.as_secs(),
                                remaining_secs = (total_wait - elapsed).as_secs(),
                                "Job still running"
                            );
                        }
                    }
                    _ = current_job.shutdown_requested() => {
                        tracing::info!(
                            job_id = %current_job.id(),
                            elapsed_secs = elapsed.as_secs(),
                            "Job received shutdown signal, exiting gracefully"
                        );
                        return Ok(JobCompletion::Complete);
                    }
                }
            } else {
                sleep(sleep_duration).await;
                elapsed += sleep_duration;
                if elapsed.as_secs() % 5 == 0 {
                    tracing::info!(
                        job_id = %current_job.id(),
                        elapsed_secs = elapsed.as_secs(),
                        remaining_secs = (total_wait - elapsed).as_secs(),
                        "Job still running (ignoring shutdown)"
                    );
                }
            }
        }

        tracing::info!(
            job_id = %current_job.id(),
            "Test wait job completed successfully"
        );

        Ok(JobCompletion::Complete)
    }
}
