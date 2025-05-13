use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::{RoleId, RoleName};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "RoleId")]
pub enum RoleEvent {
    Initialized { id: RoleId, name: RoleName },
    GainedInheritanceFrom { junior_id: RoleId },
    LostInheritanceFrom { junior_id: RoleId },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Role {
    pub id: RoleId,
    pub name: RoleName,
    events: EntityEvents<RoleEvent>,
}

impl Role {
    /// Make this role inherit from another role. Consequently, this role will
    /// gain all permissions of `junior`.
    pub(super) fn inherit_from(&mut self, junior: &Role) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            RoleEvent::GainedInheritanceFrom { junior_id } if junior.id == *junior_id,
            => RoleEvent::LostInheritanceFrom { junior_id } if junior.id == *junior_id
        );

        self.events.push(RoleEvent::GainedInheritanceFrom {
            junior_id: junior.id,
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
                RoleEvent::GainedInheritanceFrom { .. } => {}
                RoleEvent::LostInheritanceFrom { .. } => {}
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewRole {
    #[builder(setter(into))]
    pub(super) id: RoleId,
    pub(super) name: RoleName,
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
