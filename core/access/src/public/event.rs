use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::{PublicRole, PublicUser};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreAccessEvent {
    UserCreated { entity: PublicUser },
    RoleCreated { entity: PublicRole },
}
