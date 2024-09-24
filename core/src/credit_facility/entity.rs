use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{entity::*, primitives::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreditFacilityEvent {
    Initialized {
        id: CreditFacilityId,
        customer_id: CustomerId,
        facility: UsdCents,
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
    pub customer_id: CustomerId,
    pub(super) _events: EntityEvents<CreditFacilityEvent>,
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
                CreditFacilityEvent::Initialized {
                    id, customer_id, ..
                } => builder = builder.id(*id).customer_id(*customer_id),
            }
        }
        builder._events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewCreditFacility {
    #[builder(setter(into))]
    pub(super) id: CreditFacilityId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    facility: UsdCents,
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
                customer_id: self.customer_id,
                facility: self.facility,
            }],
        )
    }
}
