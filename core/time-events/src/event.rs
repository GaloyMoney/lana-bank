use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum TimeEvent {
    DailyClosing { date: NaiveDate },
}
