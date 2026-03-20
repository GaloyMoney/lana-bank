use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::public::PublicEodProcess;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreEodEvent {
    EodProcessStarted { entity: PublicEodProcess },
    EodProcessCompleted { entity: PublicEodProcess },
    EodProcessFailed { entity: PublicEodProcess },
}
