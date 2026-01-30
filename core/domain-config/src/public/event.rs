use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::DomainConfigKey;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreDomainConfigEvent {
    DomainConfigUpdated { key: DomainConfigKey },
}
