use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use audit::AuditInfo;
use es_entity::*;

use crate::{
    primitives::{RoleId, RoleName},
    PermissionSetId,
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "RoleId")]
pub enum RoleEvent {
    Initialized {
        id: RoleId,
        name: RoleName,
        permission_sets: HashSet<PermissionSetId>,
        audit_info: AuditInfo,
    },
    PermissionSetAdded {
        permission_set_id: PermissionSetId,
        audit_info: AuditInfo,
    },
    PermissionSetRemoved {
        permission_set_id: PermissionSetId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
#[allow(dead_code)]
pub struct Role {
    pub id: RoleId,
    pub name: RoleName,
    pub permission_sets: HashSet<PermissionSetId>,
    events: EntityEvents<RoleEvent>,
}

impl Role {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub(crate) fn add_permission_set(
        &mut self,
        permission_set_id: PermissionSetId,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            RoleEvent::PermissionSetAdded { permission_set_id: id, ..} if permission_set_id == *id,
            => RoleEvent::PermissionSetRemoved { permission_set_id: id, .. } if permission_set_id == *id
        );
        self.permission_sets.insert(permission_set_id);
        self.events.push(RoleEvent::PermissionSetAdded {
            permission_set_id,
            audit_info,
        });
        self.permission_sets.insert(permission_set_id);

        Idempotent::Executed(())
    }

    pub(crate) fn remove_permission_set(
        &mut self,
        permission_set_id: PermissionSetId,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            RoleEvent::PermissionSetRemoved { permission_set_id: id, .. } if permission_set_id == *id,
            => RoleEvent::PermissionSetAdded { permission_set_id: id, ..} if permission_set_id == *id
        );
        self.permission_sets.remove(&permission_set_id);
        self.events.push(RoleEvent::PermissionSetRemoved {
            permission_set_id,
            audit_info,
        });
        self.permission_sets.remove(&permission_set_id);

        Idempotent::Executed(())
    }

    pub fn permission_sets(&self) -> &HashSet<PermissionSetId> {
        &self.permission_sets
    }
}

impl TryFromEvents<RoleEvent> for Role {
    fn try_from_events(events: EntityEvents<RoleEvent>) -> Result<Self, EsEntityError> {
        let mut builder = RoleBuilder::default();
        let mut new_permission_sets = HashSet::new();

        for event in events.iter_all() {
            match event {
                RoleEvent::Initialized {
                    id,
                    name,
                    permission_sets,
                    ..
                } => {
                    new_permission_sets.extend(permission_sets);
                    builder = builder.id(*id).name(name.clone());
                }
                RoleEvent::PermissionSetAdded {
                    permission_set_id, ..
                } => {
                    new_permission_sets.insert(*permission_set_id);
                }
                RoleEvent::PermissionSetRemoved {
                    permission_set_id, ..
                } => {
                    new_permission_sets.remove(permission_set_id);
                }
            }
        }

        builder
            .permission_sets(new_permission_sets)
            .events(events)
            .build()
    }
}

#[derive(Debug, Builder)]
pub struct NewRole {
    #[builder(setter(into))]
    pub(super) id: RoleId,
    pub(super) name: RoleName,
    #[builder(default)]
    pub(super) initial_permission_sets: HashSet<PermissionSetId>,
    pub(super) audit_info: AuditInfo,
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
                permission_sets: self.initial_permission_sets,
                audit_info: self.audit_info,
            }],
        )
    }
}
