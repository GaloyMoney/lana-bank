use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::{Entity, EntityError, EntityEvent, EntityEvents},
    primitives::{AuditInfo, CreditFacilityId, InterestAccrualId, InterestAccrualIdx},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InterestAccrualEvent {
    Initialized {
        id: InterestAccrualId,
        facility_id: CreditFacilityId,
        idx: InterestAccrualIdx,
        audit_info: AuditInfo,
    },
}

impl EntityEvent for InterestAccrualEvent {
    type EntityId = InterestAccrualId;
    fn event_table_name() -> &'static str {
        "interest_accrual_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct InterestAccrual {
    pub id: InterestAccrualId,
    pub facility_id: CreditFacilityId,
    pub idx: InterestAccrualIdx,
    pub(super) _events: EntityEvents<InterestAccrualEvent>,
}

impl Entity for InterestAccrual {
    type Event = InterestAccrualEvent;
}

impl TryFrom<EntityEvents<InterestAccrualEvent>> for InterestAccrual {
    type Error = EntityError;

    fn try_from(events: EntityEvents<InterestAccrualEvent>) -> Result<Self, Self::Error> {
        let mut builder = InterestAccrualBuilder::default();
        for event in events.iter() {
            match event {
                InterestAccrualEvent::Initialized {
                    id,
                    facility_id,
                    idx,
                    ..
                } => builder = builder.id(*id).facility_id(*facility_id).idx(*idx),
            }
        }
        builder._events(events).build()
    }
}

#[derive(Debug)]
pub struct NewInterestAccrual {
    pub(super) id: InterestAccrualId,
    pub(super) facility_id: CreditFacilityId,
    pub(super) idx: InterestAccrualIdx,
    pub(super) audit_info: AuditInfo,
}

impl NewInterestAccrual {
    pub fn new(
        facility_id: CreditFacilityId,
        idx: InterestAccrualIdx,
        audit_info: AuditInfo,
    ) -> Self {
        Self {
            id: InterestAccrualId::new(),
            facility_id,
            idx,
            audit_info,
        }
    }

    pub fn initial_events(self) -> EntityEvents<InterestAccrualEvent> {
        EntityEvents::init(
            self.id,
            [InterestAccrualEvent::Initialized {
                id: self.id,
                facility_id: self.facility_id,
                idx: self.idx,
                audit_info: self.audit_info,
            }],
        )
    }
}
