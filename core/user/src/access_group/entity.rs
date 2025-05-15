use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::AccessGroupId;

type Permission = (String, String);
type Permissions = HashSet<Permission>;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AccessGroupId")]
pub enum AccessGroupEvent {
    Initialized {
        id: AccessGroupId,
        name: String,
        permissions: Permissions,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct AccessGroup {
    pub id: AccessGroupId,
    pub name: String,
    events: EntityEvents<AccessGroupEvent>,
}

impl AccessGroup {
    /// Returns all permissions assigned to this Access Group.
    pub fn permissions(&self) -> &Permissions {
        self.events
            .iter_all()
            .find_map(|event| match event {
                AccessGroupEvent::Initialized { permissions, .. } => Some(permissions),
            })
            .expect("Initialized event")
    }
}

impl TryFromEvents<AccessGroupEvent> for AccessGroup {
    fn try_from_events(events: EntityEvents<AccessGroupEvent>) -> Result<Self, EsEntityError> {
        let mut builder = AccessGroupBuilder::default();

        for event in events.iter_all() {
            match event {
                AccessGroupEvent::Initialized { id, name, .. } => {
                    builder = builder.id(*id).name(name.clone());
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewAccessGroup {
    #[builder(setter(into))]
    pub(super) id: AccessGroupId,
    pub(super) name: String,
    pub(super) permissions: Permissions,
    pub(super) audit_info: AuditInfo,
}

impl NewAccessGroup {
    pub fn builder() -> NewAccessGroupBuilder {
        Default::default()
    }
}

impl IntoEvents<AccessGroupEvent> for NewAccessGroup {
    fn into_events(self) -> EntityEvents<AccessGroupEvent> {
        EntityEvents::init(
            self.id,
            [AccessGroupEvent::Initialized {
                id: self.id,
                name: self.name,
                permissions: self.permissions,
                audit_info: self.audit_info,
            }],
        )
    }
}
