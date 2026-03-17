use async_graphql::*;

use crate::primitives::{Date, Timestamp};

#[derive(SimpleObject, Clone)]
pub struct Time {
    /// Current business date for the environment clock.
    current_date: Date,
    /// Current environment timestamp.
    current_time: Timestamp,
    /// Timestamp when the next end-of-day boundary will be reached.
    next_end_of_day_at: Timestamp,
    /// IANA timezone identifier for the environment (e.g. "America/New_York").
    timezone: String,
    /// Configured end-of-day time in HH:MM:SS format.
    end_of_day_time: String,
}

impl From<lana_app::time_events::TimeState> for Time {
    fn from(value: lana_app::time_events::TimeState) -> Self {
        Self {
            current_date: value.current_date.into(),
            current_time: value.current_time.into(),
            next_end_of_day_at: value.next_end_of_day_at.into(),
            timezone: value.timezone.to_string(),
            end_of_day_time: value.end_of_day_time.format("%H:%M:%S").to_string(),
        }
    }
}
