use serde::{Deserialize, Serialize};

use crate::primitives::RoleId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRole {
    pub id: RoleId,
    pub name: String,
}
