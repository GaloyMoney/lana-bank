use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{primitives::RoleId, role::Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicRole {
    pub id: RoleId,
    pub name: String,
}

impl From<&Role> for PublicRole {
    fn from(entity: &Role) -> Self {
        PublicRole {
            id: entity.id,
            name: entity.name.clone(),
        }
    }
}
