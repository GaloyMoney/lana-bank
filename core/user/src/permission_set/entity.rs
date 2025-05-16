use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use es_entity::*;

use crate::primitives::PermissionSetId;

type Permission = (String, String);
type Permissions = HashSet<Permission>;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PermissionSetId")]
pub enum PermissionSetEvent {
    Initialized {
        id: PermissionSetId,
        name: String,
        permissions: Permissions,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct PermissionSet {
    pub id: PermissionSetId,
    pub name: String,
    events: EntityEvents<PermissionSetEvent>,
}

impl PermissionSet {
    /// Returns all permissions assigned to this Permission Set.
    pub fn permissions(&self) -> &Permissions {
        self.events
            .iter_all()
            .find_map(|event| match event {
                PermissionSetEvent::Initialized { permissions, .. } => Some(permissions),
            })
            .expect("Initialized event")
    }
}

impl TryFromEvents<PermissionSetEvent> for PermissionSet {
    fn try_from_events(events: EntityEvents<PermissionSetEvent>) -> Result<Self, EsEntityError> {
        let mut builder = PermissionSetBuilder::default();

        for event in events.iter_all() {
            match event {
                PermissionSetEvent::Initialized { id, name, .. } => {
                    builder = builder.id(*id).name(name.clone());
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewPermissionSet {
    #[builder(setter(into))]
    pub(super) id: PermissionSetId,
    pub(super) name: String,
    pub(super) permissions: Permissions,
}

impl NewPermissionSet {
    pub fn builder() -> NewPermissionSetBuilder {
        Default::default()
    }
}

impl IntoEvents<PermissionSetEvent> for NewPermissionSet {
    fn into_events(self) -> EntityEvents<PermissionSetEvent> {
        EntityEvents::init(
            self.id,
            [PermissionSetEvent::Initialized {
                id: self.id,
                name: self.name,
                permissions: self.permissions,
            }],
        )
    }
}
