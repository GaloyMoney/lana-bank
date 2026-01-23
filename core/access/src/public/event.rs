use serde::{Deserialize, Serialize};

use super::{PublicRole, PublicUser};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum CoreAccessEvent {
    UserCreated { entity: PublicUser },
    RoleCreated { entity: PublicRole },
}
