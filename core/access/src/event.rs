use serde::{Deserialize, Serialize};

use crate::primitives::{PermissionSetId, RoleId, RoleName, UserId};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreAccessEvent {
    UserCreated {
        id: UserId,
        email: String,
    },
    UserRemoved {
        id: UserId,
    },
    UserGrantedRole {
        id: UserId,
        role: RoleName,
    },
    UserRevokedRole {
        id: UserId,
        role: RoleName,
    },

    RoleCreated {
        id: RoleId,
        name: RoleName,
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
