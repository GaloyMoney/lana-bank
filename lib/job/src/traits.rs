use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::{
    current::CurrentJob,
    entity::{Job, JobType},
    error::JobRunError,
};

pub trait JobInitializer: Send + Sync + 'static {
    fn job_type() -> JobType
    where
        Self: Sized;

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        Default::default()
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>>;
}

pub trait JobConfig: serde::Serialize {
    type Initializer: JobInitializer;
}

pub enum JobCompletion {
    Complete,
    CompleteWithOp(es_entity::DbOp<'static>),
    RescheduleNow,
    RescheduleNowWithOp(es_entity::DbOp<'static>),
    RescheduleIn(std::time::Duration),
    RescheduleInWithOp(std::time::Duration, es_entity::DbOp<'static>),
    RescheduleAt(DateTime<Utc>),
    RescheduleAtWithOp(es_entity::DbOp<'static>, DateTime<Utc>),
}

#[async_trait]
pub trait JobRunner: Send + Sync + 'static {
    async fn run(&self, current_job: CurrentJob) -> Result<JobCompletion, JobRunError>;
}

#[derive(Debug)]
pub struct RetrySettings {
    pub n_attempts: Option<u32>,
    pub n_warn_attempts: Option<u32>,
    pub min_backoff: std::time::Duration,
    pub max_backoff: std::time::Duration,
    pub backoff_jitter_pct: u32,
}

impl RetrySettings {
    pub fn repeat_indefinitely() -> Self {
        Self {
            n_attempts: None,
            n_warn_attempts: None,
            ..Default::default()
        }
    }

    pub(super) fn next_attempt_at(&self, attempt: u32) -> DateTime<Utc> {
        use rand::{rng, Rng};
        let base_backoff_ms = self.min_backoff.as_millis() * 2u128.pow(attempt - 1);
        let jitter_range =
            (base_backoff_ms as f64 * self.backoff_jitter_pct as f64 / 100.0) as i128;
        let jitter = rng().random_range(-jitter_range..=jitter_range);
        let jittered_backoff = (base_backoff_ms as i128 + jitter).max(0) as u128;
        let final_backoff = std::cmp::min(jittered_backoff, self.max_backoff.as_millis());
        crate::time::now() + std::time::Duration::from_millis(final_backoff as u64)
    }
}

impl Default for RetrySettings {
    fn default() -> Self {
        const SECS_IN_ONE_MONTH: u64 = 60 * 60 * 24 * 30;
        Self {
            n_attempts: Some(30),
            n_warn_attempts: Some(3),
            min_backoff: std::time::Duration::from_secs(1),
            max_backoff: std::time::Duration::from_secs(SECS_IN_ONE_MONTH),
            backoff_jitter_pct: 20,
        }
    }
}
