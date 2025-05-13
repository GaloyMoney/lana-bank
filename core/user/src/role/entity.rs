use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::RoleId;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "RoleId")]
pub enum RoleEvent {
    Initialized { id: RoleId, name: String },
    AssignedToParent { id: RoleId, parent: RoleId },
    RemovedFromParent { id: RoleId, parent: RoleId },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Role {
    pub id: RoleId,
    pub name: String,
    events: EntityEvents<RoleEvent>,
}

impl Role {
    pub(super) fn assign_to_parent(&mut self, parent: &Role) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            RoleEvent::AssignedToParent { parent: parent_id, .. } if parent.id == *parent_id,
            => RoleEvent::RemovedFromParent { parent: parent_id, .. } if parent.id == *parent_id
        );

        self.events.push(RoleEvent::AssignedToParent {
            id: self.id,
            parent: parent.id,
        });
        Idempotent::Executed(())
    }
}

impl TryFromEvents<RoleEvent> for Role {
    fn try_from_events(events: EntityEvents<RoleEvent>) -> Result<Self, EsEntityError> {
        let mut builder = RoleBuilder::default();

        for event in events.iter_all() {
            match event {
                RoleEvent::Initialized { id, name } => {
                    builder = builder.id(*id).name(name.clone());
                }
                _ => {}
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewRole {
    #[builder(setter(into))]
    pub(super) id: RoleId,
    #[builder(setter(into))]
    pub(super) name: String,
}

impl NewRole {
    pub fn builder() -> NewRoleBuilder {
        Default::default()
    }
}

impl IntoEvents<RoleEvent> for NewRole {
    fn into_events(self) -> EntityEvents<RoleEvent> {
        EntityEvents::init(
            self.id,
            [RoleEvent::Initialized {
                id: self.id,
                name: self.name,
            }],
        )
    }
}
