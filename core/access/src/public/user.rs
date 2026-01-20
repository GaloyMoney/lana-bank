use chrono::{DateTime, Utc};
use es_entity::PersistedEvent;
use serde::{Deserialize, Serialize};

use crate::{
    primitives::{RoleId, UserId},
    user::{User, UserEvent},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: UserId,
    pub email: String,
    pub role_id: RoleId,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl From<(&User, &PersistedEvent<UserEvent>)> for PublicUser {
    fn from((entity, event): (&User, &PersistedEvent<UserEvent>)) -> Self {
        PublicUser {
            id: entity.id,
            email: entity.email.clone(),
            role_id: entity.current_role(),
            created_at: event.recorded_at,
            created_by: super::extract_sub(&event.context),
        }
    }
}
