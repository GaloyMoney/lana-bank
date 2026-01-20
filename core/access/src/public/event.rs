use serde::{Deserialize, Serialize};

use crate::primitives::{RoleId, UserId};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum CoreAccessEvent {
    UserCreated {
        id: UserId,
        email: String,
        role_id: RoleId,
    },
    RoleCreated {
        id: RoleId,
        name: String,
    },
}
