use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::{Entity, EntityError, EntityEvent, EntityEvents},
    primitives::{AuditInfo, LoanId, UnaccruedInterestId, UnaccruedInterestIdx},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnaccruedInterestEvent {
    Initialized {
        id: UnaccruedInterestId,
        loan_id: LoanId,
        idx: UnaccruedInterestIdx,
        audit_info: AuditInfo,
    },
}

impl EntityEvent for UnaccruedInterestEvent {
    type EntityId = UnaccruedInterestId;
    fn event_table_name() -> &'static str {
        "unaccrued_interest_events"
    }
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct UnaccruedInterest {
    pub id: UnaccruedInterestId,
    pub loan_id: LoanId,
    pub idx: UnaccruedInterestIdx,
    pub(super) _events: EntityEvents<UnaccruedInterestEvent>,
}

impl Entity for UnaccruedInterest {
    type Event = UnaccruedInterestEvent;
}

impl TryFrom<EntityEvents<UnaccruedInterestEvent>> for UnaccruedInterest {
    type Error = EntityError;

    fn try_from(events: EntityEvents<UnaccruedInterestEvent>) -> Result<Self, Self::Error> {
        let mut builder = UnaccruedInterestBuilder::default();
        for event in events.iter() {
            match event {
                UnaccruedInterestEvent::Initialized {
                    id, loan_id, idx, ..
                } => builder = builder.id(*id).loan_id(*loan_id).idx(*idx),
            }
        }
        builder._events(events).build()
    }
}

#[derive(Debug)]
pub struct NewUnaccruedInterest {
    pub(super) id: UnaccruedInterestId,
    pub(super) loan_id: LoanId,
    pub(super) idx: UnaccruedInterestIdx,
    pub(super) audit_info: AuditInfo,
}

impl NewUnaccruedInterest {
    pub fn new(loan_id: LoanId, idx: UnaccruedInterestIdx, audit_info: AuditInfo) -> Self {
        Self {
            id: UnaccruedInterestId::new(),
            loan_id,
            idx,
            audit_info,
        }
    }

    pub fn initial_events(self) -> EntityEvents<UnaccruedInterestEvent> {
        EntityEvents::init(
            self.id,
            [UnaccruedInterestEvent::Initialized {
                id: self.id,
                loan_id: self.loan_id,
                idx: self.idx,
                audit_info: self.audit_info,
            }],
        )
    }
}
