use serde::{Deserialize, Serialize};

use crate::primitives::{RoleId, RoleName, UserId};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreUserEvent {
    UserCreated { id: UserId, email: String },
    UserRemoved { id: UserId },
    UserGainedRole { user_id: UserId, role: RoleName },
    UserLostRole { user_id: UserId, role: RoleName },

    RoleCreated { id: RoleId, name: String },
}
