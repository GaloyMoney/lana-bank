use serde::{Deserialize, Serialize};

use crate::primitives::{RoleId, UserId};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreUserEvent {
    UserCreated { id: UserId, email: String },
    UserRemoved { id: UserId },
    RoleCreated { id: RoleId, name: String },
}
