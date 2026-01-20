use audit::AuditInfo;
use chrono::{DateTime, Utc};
use es_entity::PersistedEvent;
use serde::{Deserialize, Serialize};

use crate::{
    primitives::RoleId,
    role::{Role, RoleEvent},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRole {
    pub id: RoleId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl From<(&Role, &PersistedEvent<RoleEvent>)> for PublicRole {
    fn from((entity, event): (&Role, &PersistedEvent<RoleEvent>)) -> Self {
        PublicRole {
            id: entity.id,
            name: entity.name.clone(),
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
