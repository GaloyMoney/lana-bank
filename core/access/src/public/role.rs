use serde::{Deserialize, Serialize};

use crate::{primitives::RoleId, role::Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
