use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreTimeEvent {
    EndOfDay { closing_time: DateTime<Utc> },
}
