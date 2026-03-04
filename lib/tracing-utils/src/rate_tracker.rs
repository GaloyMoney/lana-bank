use std::{
    collections::{HashMap, VecDeque},
    sync::{Mutex, OnceLock},
    time::Instant,
};

static TRACKER: OnceLock<Mutex<HashMap<String, VecDeque<Instant>>>> = OnceLock::new();

fn tracker() -> &'static Mutex<HashMap<String, VecDeque<Instant>>> {
    TRACKER.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Records an error occurrence and returns whether the aggregate alert threshold
/// has been exceeded within the given time window.
///
/// - `key`: unique identifier for this error source (e.g., `module::fn_name::VariantName`)
/// - `threshold`: number of errors required to trigger (0 = disabled, always returns false)
/// - `window_secs`: sliding window duration in seconds
///
/// Returns `true` if the number of occurrences within the window >= threshold.
pub fn should_trigger_aggregate_alert(key: &str, threshold: u64, window_secs: u64) -> bool {
    if threshold == 0 {
        return false;
    }

    let now = Instant::now();
    let window = std::time::Duration::from_secs(window_secs);

    let mut map = tracker().lock().expect("rate tracker lock poisoned");
    let entries = map.entry(key.to_string()).or_default();

    // Evict entries outside the window
    while let Some(front) = entries.front() {
        if now.duration_since(*front) > window {
            entries.pop_front();
        } else {
            break;
        }
    }

    // Record current occurrence
    entries.push_back(now);

    entries.len() as u64 >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_threshold_never_triggers() {
        assert!(!should_trigger_aggregate_alert("test::disabled", 0, 600));
    }

    #[test]
    fn triggers_at_threshold() {
        let key = "test::triggers_at_threshold";
        for i in 0..9 {
            assert!(
                !should_trigger_aggregate_alert(key, 10, 600),
                "should not trigger at count {}",
                i + 1
            );
        }
        assert!(
            should_trigger_aggregate_alert(key, 10, 600),
            "should trigger at count 10"
        );
    }

    #[test]
    fn threshold_one_triggers_immediately() {
        assert!(should_trigger_aggregate_alert(
            "test::threshold_one",
            1,
            600
        ));
    }
}
