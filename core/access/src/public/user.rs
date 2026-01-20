use audit::AuditInfo;
use chrono::{DateTime, Utc};
use es_entity::PersistedEvent;
use serde::{Deserialize, Serialize};

use crate::{
    primitives::{RoleId, UserId},
    user::UserEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: UserId,
    pub email: String,
    pub role_id: RoleId,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl TryFrom<&PersistedEvent<UserEvent>> for PublicUser {
    type Error = ();

    fn try_from(event: &PersistedEvent<UserEvent>) -> Result<Self, Self::Error> {
        match &event.event {
            UserEvent::Initialized {
                id, email, role_id, ..
            } => Ok(PublicUser {
                id: *id,
                email: email.clone(),
                role_id: *role_id,
                created_at: event.recorded_at,
                created_by: extract_created_by(&event.context),
            }),
            _ => Err(()),
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
