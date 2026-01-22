use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreTimeEvent {
    EndOfDay {
        day: NaiveDate,
        closing_time: DateTime<Utc>,
        timezone: chrono_tz::Tz,
    },
}
