use std::collections::HashSet;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use audit::AuditInfo;

use crate::primitives::{CommitteeId, UserId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CommitteeId")]
pub enum CommitteeEvent {
    Initialized {
        id: CommitteeId,
        name: String,
        audit_info: AuditInfo,
    },
    UserAdded {
        user_id: UserId,
        audit_info: AuditInfo,
    },
    UserRemoved {
        user_id: UserId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Committee {
    pub id: CommitteeId,
    pub name: String,
    pub(super) events: EntityEvents<CommitteeEvent>,
}

impl Committee {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for committee")
    }

    fn is_user_added(&self, user_id: UserId) -> bool {
        for event in self.events.iter_all() {
            if let CommitteeEvent::UserAdded { user_id: id, .. } = event {
                if *id == user_id {
                    return true;
                }
            }
        }
        false
    }

    pub fn add_user(&mut self, user_id: UserId, audit_info: AuditInfo) {
        if self.is_user_added(user_id) {
            return;
        }

        self.events.push(CommitteeEvent::UserAdded {
            user_id,
            audit_info,
        });
    }

    pub fn remove_user(&mut self, user_id: UserId, audit_info: AuditInfo) {
        if !self.is_user_added(user_id) {
            return;
        }
        self.events.push(CommitteeEvent::UserRemoved {
            user_id,
            audit_info,
        });
    }

    pub fn users(&self) -> HashSet<UserId> {
        let mut users = HashSet::new();

        for event in self.events.iter_all() {
            match event {
                CommitteeEvent::UserAdded { user_id, .. } => {
                    users.insert(*user_id);
                }
                CommitteeEvent::UserRemoved { user_id, .. } => {
                    users.remove(user_id);
                }
                _ => {}
            }
        }
        users
    }
}

impl TryFromEvents<CommitteeEvent> for Committee {
    fn try_from_events(events: EntityEvents<CommitteeEvent>) -> Result<Self, EsEntityError> {
        let mut builder = CommitteeBuilder::default();
        for event in events.iter_all() {
            match event {
                CommitteeEvent::Initialized { id, name, .. } => {
                    builder = builder.id(*id).name(name.clone())
                }
                CommitteeEvent::UserAdded { .. } => {}
                CommitteeEvent::UserRemoved { .. } => {}
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewCommittee {
    #[builder(setter(into))]
    pub(super) id: CommitteeId,
    pub(super) name: String,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewCommittee {
    pub fn builder() -> NewCommitteeBuilder {
        NewCommitteeBuilder::default()
    }
}

impl IntoEvents<CommitteeEvent> for NewCommittee {
    fn into_events(self) -> EntityEvents<CommitteeEvent> {
        EntityEvents::init(
            self.id,
            [CommitteeEvent::Initialized {
                id: self.id,
                name: self.name,
                audit_info: self.audit_info,
            }],
        )
    }
}
