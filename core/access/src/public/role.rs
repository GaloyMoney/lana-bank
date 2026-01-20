use audit::AuditInfo;
use chrono::{DateTime, Utc};
use es_entity::PersistedEvent;
use serde::{Deserialize, Serialize};

use crate::{primitives::RoleId, role::RoleEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRole {
    pub id: RoleId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl TryFrom<&PersistedEvent<RoleEvent>> for PublicRole {
    type Error = ();

    fn try_from(event: &PersistedEvent<RoleEvent>) -> Result<Self, Self::Error> {
        match &event.event {
            RoleEvent::Initialized { id, name, .. } => Ok(PublicRole {
                id: *id,
                name: name.clone(),
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
