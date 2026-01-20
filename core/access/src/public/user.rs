use audit::AuditInfo;
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
            created_by: extract_created_by(&event.context),
        }
    }
}

fn extract_created_by(context: &Option<es_entity::ContextData>) -> String {
    context
        .as_ref()
        .and_then(|ctx| ctx.lookup::<AuditInfo>("audit_info").ok().flatten())
        .map(|info| info.sub)
        .unwrap_or_else(|| "unknown".to_string())
}
