use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    primitives::{RoleId, UserId},
    user::User,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicUser {
    pub id: UserId,
    pub email: String,
    pub role_id: RoleId,
}

impl From<&User> for PublicUser {
    fn from(entity: &User) -> Self {
        PublicUser {
            id: entity.id,
            email: entity.email.clone(),
            role_id: entity.current_role(),
        }
    }
}
