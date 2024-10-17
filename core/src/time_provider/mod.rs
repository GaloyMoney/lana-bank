use chrono::{DateTime, Duration, Utc};

pub trait TimeProvider {
    fn now(&self) -> DateTime<Utc>;
}

pub struct RealTimeProvider;

impl TimeProvider for RealTimeProvider {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub struct MockTimeProvider {
    start_time: DateTime<Utc>,
    delta: Duration,
}

impl MockTimeProvider {
    pub fn new() -> Self {
        Self {
            start_time: Utc::now(),
            delta: Duration::zero(),
        }
    }

    pub fn with_delta(delta: Duration) -> Self {
        Self {
            start_time: Utc::now(),
            delta,
        }
    }

    pub fn set_delta(&mut self, delta: Duration) {
        self.delta = delta;
    }

    pub fn advance_time(&mut self, duration: Duration) {
        self.delta += duration;
    }

    pub fn regress_time(&mut self, duration: Duration) {
        self.delta -= duration;
    }
}

impl TimeProvider for MockTimeProvider {
    fn now(&self) -> DateTime<Utc> {
        self.start_time + self.delta
    }
}

impl Default for MockTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    const EPSILON: i64 = 100; // milliseconds

    fn within_epsilon(a: DateTime<Utc>, b: DateTime<Utc>) -> bool {
        (a - b).num_milliseconds().abs() <= EPSILON
    }

    #[test]
    fn test_new_mock_provider() {
        let mock = MockTimeProvider::new();
        let now = Utc::now();
        assert!(within_epsilon(mock.now(), now));
    }

    #[test]
    fn test_with_delta() {
        let delta = Duration::days(5);
        let mock = MockTimeProvider::with_delta(delta);
        let expected = Utc::now() + delta;
        assert!(within_epsilon(mock.now(), expected));
    }

    #[test]
    fn test_set_delta() {
        let mut mock = MockTimeProvider::new();
        let delta = Duration::hours(3);
        mock.set_delta(delta);
        let expected = Utc::now() + delta;
        assert!(within_epsilon(mock.now(), expected));
    }

    #[test]
    fn test_advance_time() {
        let mut mock = MockTimeProvider::new();
        let initial_time = mock.now();
        let advance_duration = Duration::minutes(30);
        mock.advance_time(advance_duration);
        assert!(within_epsilon(mock.now(), initial_time + advance_duration));
    }

    #[test]
    fn test_regress_time() {
        let mut mock = MockTimeProvider::new();
        let initial_time = mock.now();
        let regress_duration = Duration::hours(2);
        mock.regress_time(regress_duration);
        assert!(within_epsilon(mock.now(), initial_time - regress_duration));
    }

    #[test]
    fn test_multiple_time_changes() {
        let mut mock = MockTimeProvider::new();
        let initial_time = mock.now();

        mock.advance_time(Duration::hours(5));
        mock.regress_time(Duration::hours(2));
        mock.advance_time(Duration::minutes(30));

        let expected = initial_time + Duration::hours(3) + Duration::minutes(30);
        assert!(within_epsilon(mock.now(), expected));
    }

    #[test]
    fn test_consistency_over_multiple_calls() {
        let mock = MockTimeProvider::with_delta(Duration::days(1));
        let first_call = mock.now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let second_call = mock.now();
        assert_eq!(first_call, second_call);
    }

    #[test]
    fn test_negative_delta() {
        let mut mock = MockTimeProvider::new();
        let negative_delta = Duration::hours(-5);
        mock.set_delta(negative_delta);
        let expected = Utc::now() + negative_delta;
        assert!(within_epsilon(mock.now(), expected));
    }
}
