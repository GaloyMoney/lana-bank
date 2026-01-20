use serde::{Deserialize, Serialize};

use crate::primitives::{RoleId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: UserId,
    pub email: String,
    pub role_id: RoleId,
}
