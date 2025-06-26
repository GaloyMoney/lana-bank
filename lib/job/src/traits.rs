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
        use rand::{Rng, rng};

        // Cap the attempt number to prevent exponential overflow
        // 2^30 = ~1 billion, which is already a very large multiplier
        let capped_attempt = std::cmp::min(attempt.saturating_sub(1), 30);

        let base_backoff_ms = self.min_backoff.as_millis();

        // Use checked arithmetic to prevent overflow
        let exponential_multiplier = 2u128.saturating_pow(capped_attempt);
        let exponential_backoff = base_backoff_ms.saturating_mul(exponential_multiplier);

        // Cap at max_backoff early to prevent further overflow
        let capped_backoff = std::cmp::min(exponential_backoff, self.max_backoff.as_millis());

        let jitter_range = (capped_backoff as f64 * self.backoff_jitter_pct as f64 / 100.0) as i128;
        let jitter = rng().random_range(-jitter_range..=jitter_range);

        // Use saturating arithmetic to prevent overflow
        let jittered_backoff = (capped_backoff as i128).saturating_add(jitter).max(0) as u128;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_retry_settings() -> RetrySettings {
        RetrySettings {
            n_attempts: Some(10),
            n_warn_attempts: Some(3),
            min_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(60), // 60 seconds = 60,000 ms
            backoff_jitter_pct: 0,                // No jitter for predictable testing
        }
    }

    fn test_retry_settings_with_jitter() -> RetrySettings {
        RetrySettings {
            n_attempts: Some(10),
            n_warn_attempts: Some(3),
            min_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(60),
            backoff_jitter_pct: 20, // 20% jitter
        }
    }

    #[test]
    fn test_next_attempt_at_basic_exponential_growth() {
        let settings = test_retry_settings();
        let now = crate::time::now();

        // Test basic exponential growth: 2^0, 2^1, 2^2, 2^3
        let attempt1_time = settings.next_attempt_at(1);
        let attempt2_time = settings.next_attempt_at(2);
        let attempt3_time = settings.next_attempt_at(3);
        let attempt4_time = settings.next_attempt_at(4);

        // Calculate expected delays (in milliseconds)
        // attempt 1: 100ms * 2^0 = 100ms
        // attempt 2: 100ms * 2^1 = 200ms
        // attempt 3: 100ms * 2^2 = 400ms
        // attempt 4: 100ms * 2^3 = 800ms

        let delay1 = attempt1_time.signed_duration_since(now).num_milliseconds();
        let delay2 = attempt2_time.signed_duration_since(now).num_milliseconds();
        let delay3 = attempt3_time.signed_duration_since(now).num_milliseconds();
        let delay4 = attempt4_time.signed_duration_since(now).num_milliseconds();

        assert_eq!(delay1, 100);
        assert_eq!(delay2, 200);
        assert_eq!(delay3, 400);
        assert_eq!(delay4, 800);
    }

    #[test]
    fn test_next_attempt_at_zero_attempt_edge_case() {
        let settings = test_retry_settings();
        let now = crate::time::now();

        // Attempt 0 should not panic and should use saturating_sub
        let attempt_time = settings.next_attempt_at(0);
        let delay = attempt_time.signed_duration_since(now).num_milliseconds();

        // With attempt 0, saturating_sub(1) = 0, so 2^0 = 1
        // Expected: 100ms * 1 = 100ms
        assert_eq!(delay, 100);
    }

    #[test]
    fn test_next_attempt_at_max_backoff_capping() {
        let settings = test_retry_settings();
        let now = crate::time::now();

        // Test with a high attempt number that would exceed max_backoff
        // attempt 20: 100ms * 2^19 = 100ms * 524,288 = 52,428,800ms = ~14.5 hours
        // But max_backoff is 60,000ms (60 seconds), so it should be capped
        let attempt_time = settings.next_attempt_at(20);
        let delay = attempt_time.signed_duration_since(now).num_milliseconds();

        assert_eq!(delay, 60_000); // Should be capped at max_backoff
    }

    #[test]
    fn test_next_attempt_at_overflow_protection() {
        let settings = test_retry_settings();
        let now = crate::time::now();

        // Test with extremely high attempt numbers that would cause overflow
        // These should be capped at 30 due to our overflow protection
        for extreme_attempt in [100, 1000, u32::MAX] {
            let attempt_time = settings.next_attempt_at(extreme_attempt);
            let delay = attempt_time.signed_duration_since(now).num_milliseconds();

            // Should be capped at max_backoff (60,000ms) due to overflow protection
            assert_eq!(delay, 60_000);
        }
    }

    #[test]
    fn test_next_attempt_at_attempt_capping_at_30() {
        let settings = test_retry_settings();
        let now = crate::time::now();

        // Test that attempts are capped at 30 (2^30 is the limit)
        let attempt31_time = settings.next_attempt_at(31);
        let attempt100_time = settings.next_attempt_at(100);

        let delay31 = attempt31_time.signed_duration_since(now).num_milliseconds();
        let delay100 = attempt100_time
            .signed_duration_since(now)
            .num_milliseconds();

        // Both should be the same because both are capped at attempt 30
        assert_eq!(delay31, delay100);
        assert_eq!(delay31, 60_000); // Both should hit max_backoff
    }

    #[test]
    fn test_next_attempt_at_with_jitter() {
        let settings = test_retry_settings_with_jitter();
        let now = crate::time::now();

        // With jitter, we can't predict exact values, but we can test ranges
        let attempt_time = settings.next_attempt_at(1);
        let delay = attempt_time.signed_duration_since(now).num_milliseconds();

        // Base delay for attempt 1: 100ms
        // With 20% jitter: range should be 80ms to 120ms
        assert!(
            delay >= 80 && delay <= 120,
            "Delay {} not in expected jitter range 80-120",
            delay
        );
    }

    #[test]
    fn test_next_attempt_at_jitter_never_negative() {
        let settings = test_retry_settings_with_jitter();
        let now = crate::time::now();

        // Test multiple times to ensure jitter never makes delay negative
        for _ in 0..10 {
            let attempt_time = settings.next_attempt_at(1);
            let delay = attempt_time.signed_duration_since(now).num_milliseconds();
            assert!(delay >= 0, "Delay should never be negative, got {}", delay);
        }
    }

    #[test]
    fn test_next_attempt_at_deterministic_without_jitter() {
        let settings = test_retry_settings();

        // Without jitter, results should be deterministic
        let time1 = settings.next_attempt_at(5);
        let time2 = settings.next_attempt_at(5);

        // Times should be very close (within a few milliseconds due to time::now() calls)
        let diff = (time1.signed_duration_since(time2))
            .num_milliseconds()
            .abs();
        assert!(
            diff <= 5,
            "Times should be nearly identical without jitter, diff: {}ms",
            diff
        );
    }
}
