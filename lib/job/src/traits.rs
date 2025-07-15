use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::{
    current::CurrentJob,
    entity::{Job, JobType},
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
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>>;
}

#[derive(Debug)]
pub struct RetrySettings {
    pub n_attempts: Option<u32>,
    pub n_warn_attempts: Option<u32>,
    pub min_backoff: std::time::Duration,
    pub max_backoff: std::time::Duration,
    pub backoff_jitter_pct: u8,
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
        let backoff_ms = self.calculate_backoff(attempt);
        crate::time::now() + std::time::Duration::from_millis(backoff_ms)
    }

    fn calculate_backoff(&self, attempt: u32) -> u64 {
        // Calculate base exponential backoff with overflow protection
        let safe_attempt = attempt.saturating_sub(1).min(30);
        let base_ms = self.min_backoff.as_millis() as u64;
        let max_ms = self.max_backoff.as_millis() as u64;

        // Use u64 arithmetic with saturation to prevent overflow
        let backoff = base_ms.saturating_mul(1u64 << safe_attempt).min(max_ms);

        // Apply jitter if configured
        if self.backoff_jitter_pct == 0 {
            backoff
        } else {
            self.apply_jitter(backoff, max_ms)
        }
    }

    fn apply_jitter(&self, backoff_ms: u64, max_ms: u64) -> u64 {
        use rand::{Rng, rng};

        let jitter_amount = backoff_ms * self.backoff_jitter_pct as u64 / 100;
        let jitter = rng().random_range(-(jitter_amount as i64)..=(jitter_amount as i64));

        let jittered = (backoff_ms as i64 + jitter).max(0) as u64;
        jittered.min(max_ms)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const MAX_BACKOFF_MS: u64 = 60_000;
    const TIMING_TOLERANCE_MS: u64 = 5;

    fn test_settings(jitter_pct: u8) -> RetrySettings {
        RetrySettings {
            n_attempts: Some(10),
            n_warn_attempts: Some(3),
            min_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(60),
            backoff_jitter_pct: jitter_pct,
        }
    }

    fn get_delay_ms(settings: &RetrySettings, attempt: u32) -> u64 {
        let now = crate::time::now();
        let attempt_time = settings.next_attempt_at(attempt);
        attempt_time.signed_duration_since(now).num_milliseconds() as u64
    }

    fn assert_delay_exact(actual: u64, expected: u64) {
        assert_eq!(
            actual, expected,
            "Expected exactly {expected}ms, got {actual}ms"
        );
    }

    fn assert_delay_near(actual: u64, expected: u64) {
        let diff = actual.abs_diff(expected);
        assert!(
            diff <= TIMING_TOLERANCE_MS,
            "Expected ~{expected}ms (±{TIMING_TOLERANCE_MS}ms), got {actual}ms"
        );
    }

    fn assert_delay_in_range(actual: u64, min: u64, max: u64) {
        assert!(
            actual >= min && actual <= max,
            "Expected delay in range {min}-{max}ms, got {actual}ms"
        );
    }

    #[test]
    fn exponential_backoff_grows_correctly() {
        let settings = test_settings(0);
        let expected_delays = [100, 200, 400, 800]; // 100ms * 2^(n-1)

        for (attempt, &expected) in (1..=4).zip(&expected_delays) {
            let actual = settings.calculate_backoff(attempt);
            assert_delay_exact(actual, expected);
        }
    }

    #[test]
    fn zero_attempt_handled_correctly() {
        let settings = test_settings(0);
        let delay = settings.calculate_backoff(0);
        assert_delay_exact(delay, 100); // saturating_sub(1) = 0, so 2^0 = 1
    }

    #[test]
    fn high_attempts_capped_at_max_backoff() {
        let settings = test_settings(0);

        for high_attempt in [20, 31, 100, 1000, u32::MAX] {
            let delay = settings.calculate_backoff(high_attempt);
            assert_delay_exact(delay, MAX_BACKOFF_MS);
        }
    }

    #[test]
    fn attempts_capped_at_30() {
        let settings = test_settings(0);
        let backoff31 = settings.calculate_backoff(31);
        let backoff100 = settings.calculate_backoff(100);

        assert_eq!(backoff31, backoff100, "Both should be capped at attempt 30");
        assert_eq!(backoff31, MAX_BACKOFF_MS);
    }

    #[test]
    fn jitter_adds_randomness() {
        let settings = test_settings(20);
        let delay = settings.calculate_backoff(1);
        // Base: 100ms, 20% jitter: 80-120ms range
        assert_delay_in_range(delay, 80, 120);
    }

    #[test]
    fn jitter_never_negative() {
        let settings = test_settings(20);

        for _ in 0..10 {
            let delay = settings.calculate_backoff(1);
            assert!(delay >= 0, "Delay should be non-negative, got {delay}ms");
            assert!(delay <= 120, "Delay should be reasonable, got {delay}ms");
        }
    }

    #[test]
    fn deterministic_without_jitter() {
        let settings = test_settings(0);

        let backoff1 = settings.calculate_backoff(5);
        let backoff2 = settings.calculate_backoff(5);

        assert_eq!(
            backoff1, backoff2,
            "Backoffs should be identical without jitter"
        );
    }
}
