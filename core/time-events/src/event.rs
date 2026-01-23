use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreTimeEvent {
    EndOfDay {
        day: NaiveDate,
        closing_time: DateTime<Utc>,
        #[cfg_attr(feature = "json-schema", schemars(with = "String"))]
        timezone: chrono_tz::Tz,
    },
}
