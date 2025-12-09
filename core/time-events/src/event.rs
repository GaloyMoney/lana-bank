use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum CoreTimeEvent {
    DailyClosing { closing_time: DateTime<Utc> },
}
