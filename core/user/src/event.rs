use serde::{Deserialize, Serialize};

use crate::primitives::{RoleId, RoleName, UserId};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreUserEvent {
    UserCreated { user_id: UserId, email: String },
    UserRemoved { user_id: UserId },
    RoleAssigned { user_id: UserId, role: RoleName },
    RoleRevoked { user_id: UserId, role: RoleName },
    RoleCreated { role_id: RoleId, name: String },
}
