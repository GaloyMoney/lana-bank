use chrono::{DateTime, Utc};
use derive_builder::Builder;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{entity::*, primitives::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreditFacilityEvent {
    Initialized {
        id: CreditFacilityId,
        audit_info: AuditInfo,
    },
}

impl EntityEvent for CreditFacilityEvent {
    type EntityId = CreditFacilityId;
    fn event_table_name() -> &'static str {
        "credit_facility_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct CreditFacility {
    pub id: CreditFacilityId,
    pub(super) events: EntityEvents<CreditFacilityEvent>,
}

impl Entity for CreditFacility {
    type Event = CreditFacilityEvent;
}

impl CreditFacility {}

impl TryFrom<EntityEvents<CreditFacilityEvent>> for CreditFacility {
    type Error = EntityError;

    fn try_from(events: EntityEvents<CreditFacilityEvent>) -> Result<Self, Self::Error> {
        let mut builder = CreditFacilityBuilder::default();
        for event in events.iter() {
            match event {
                CreditFacilityEvent::Initialized { id, .. } => builder = builder.id(*id),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewCreditFacility {
    #[builder(setter(into))]
    pub(super) id: CreditFacilityId,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
}

impl NewCreditFacility {
    pub fn builder() -> NewCreditFacilityBuilder {
        NewCreditFacilityBuilder::default()
    }

    pub(super) fn initial_events(self) -> EntityEvents<CreditFacilityEvent> {
        EntityEvents::init(
            self.id,
            [CreditFacilityEvent::Initialized {
                id: self.id,
                audit_info: self.audit_info,
            }],
        )
    }
}
