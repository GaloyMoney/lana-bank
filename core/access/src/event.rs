use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::primitives::{PermissionSetId, RoleId, UserId};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreAccessEvent {
    UserCreated {
        id: UserId,
        email: String,
        role_id: RoleId,
    },
    UserRemoved {
        id: UserId,
    },
    UserUpdatedRole {
        id: UserId,
        role_id: RoleId,
    },

    RoleCreated {
        id: RoleId,
        name: String,
    },
    RoleGainedPermissionSet {
        id: RoleId,
        permission_set_id: PermissionSetId,
    },
    RoleLostPermissionSet {
        id: RoleId,
        permission_set_id: PermissionSetId,
    },
}
