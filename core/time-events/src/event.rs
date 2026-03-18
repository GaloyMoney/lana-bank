use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// TODO: CoreTimeEvent is no longer published in production (the EOD producer now
// spawns the process-manager job directly). It is still referenced in generic
// OutboxEventMarker<CoreTimeEvent> bounds across 10+ files and in test helpers
// that publish EndOfDay events. Remove those bounds and migrate tests to use
// the new PM-based flow, then delete this type.
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
