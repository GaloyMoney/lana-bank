use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
// TODO: use json-schema flag and add macro?
pub enum CoreTimeEvent {
    EndOfDay { closing_time: DateTime<Utc> },
}
