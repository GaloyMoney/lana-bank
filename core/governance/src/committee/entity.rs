use std::collections::HashSet;

use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::{CommitteeId, CommitteeMemberId};

use super::error::CommitteeError;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CommitteeId")]
pub enum CommitteeEvent {
    Initialized { id: CommitteeId, name: String },
    MemberAdded { member_id: CommitteeMemberId },
    MemberRemoved { member_id: CommitteeMemberId },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Committee {
    pub id: CommitteeId,
    pub name: String,
    events: EntityEvents<CommitteeEvent>,
}

impl Committee {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for committee")
    }

    pub(crate) fn add_member(
        &mut self,
        member_id: CommitteeMemberId,
    ) -> Result<Idempotent<()>, CommitteeError> {
        if self.members().contains(&member_id) {
            return Ok(Idempotent::AlreadyApplied);
        }

        self.events.push(CommitteeEvent::MemberAdded { member_id });

        Ok(Idempotent::Executed(()))
    }

    pub(crate) fn remove_member(&mut self, member_id: CommitteeMemberId) -> Idempotent<()> {
        if !self.members().contains(&member_id) {
            return Idempotent::AlreadyApplied;
        }
        self.events
            .push(CommitteeEvent::MemberRemoved { member_id });
        Idempotent::Executed(())
    }

    pub fn n_members(&self) -> usize {
        self.events.iter_all().fold(0, |count, event| match event {
            CommitteeEvent::MemberAdded { .. } => count + 1,
            CommitteeEvent::MemberRemoved { .. } => count - 1,
            _ => count,
        })
    }

    pub fn members(&self) -> HashSet<CommitteeMemberId> {
        let mut members = HashSet::new();

        for event in self.events.iter_all() {
            match event {
                CommitteeEvent::MemberAdded { member_id, .. } => {
                    members.insert(*member_id);
                }
                CommitteeEvent::MemberRemoved { member_id, .. } => {
                    members.remove(member_id);
                }
                _ => {}
            }
        }
        members
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
                CommitteeEvent::MemberAdded { .. } => {}
                CommitteeEvent::MemberRemoved { .. } => {}
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
            }],
        )
    }
}
