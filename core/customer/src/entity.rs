use chrono::{DateTime, Utc};
use derive_builder::Builder;

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::error::CustomerError;
use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CustomerId")]
pub enum CustomerEvent {
    Initialized {
        id: CustomerId,
        party_id: PartyId,
        customer_type: CustomerType,
        public_id: PublicId,
        conversion: CustomerConversion,
        level: KycLevel,
    },
    Frozen {
        status: CustomerStatus,
    },
    Unfrozen {
        status: CustomerStatus,
    },
    Closed {
        status: CustomerStatus,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityHydrationError"))]
pub struct Customer {
    pub id: CustomerId,
    pub party_id: PartyId,
    pub customer_type: CustomerType,
    pub status: CustomerStatus,
    pub level: KycLevel,
    pub conversion: CustomerConversion,
    pub public_id: PublicId,
    events: EntityEvents<CustomerEvent>,
}

impl core::fmt::Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Customer: {}", self.id)
    }
}

impl Customer {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn has_been_verified(&self) -> bool {
        matches!(self.conversion, CustomerConversion::SumsubApproved { .. })
    }

    pub fn applicant_id(&self) -> Option<&str> {
        self.conversion.applicant_id()
    }

    pub fn may_attach_product(&self, require_verified: bool) -> bool {
        !self.is_closed() && !self.is_frozen() && (!require_verified || self.has_been_verified())
    }

    pub fn is_closed(&self) -> bool {
        self.status == CustomerStatus::Closed
    }

    pub fn should_sync_financial_transactions(&self) -> bool {
        self.has_been_verified()
    }

    pub(crate) fn close(&mut self) -> Idempotent<()> {
        idempotency_guard!(self.events.iter_all().rev(), CustomerEvent::Closed { .. });
        let status = CustomerStatus::Closed;
        self.events.push(CustomerEvent::Closed { status });
        self.status = status;
        Idempotent::Executed(())
    }

    pub fn is_frozen(&self) -> bool {
        self.status == CustomerStatus::Frozen
    }

    pub fn freeze(&mut self) -> Result<Idempotent<()>, CustomerError> {
        if self.is_closed() {
            return Err(CustomerError::CustomerIsClosed);
        }
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::Frozen { .. },
            => CustomerEvent::Unfrozen { .. }
        );
        let status = CustomerStatus::Frozen;
        self.events.push(CustomerEvent::Frozen { status });
        self.status = status;
        Ok(Idempotent::Executed(()))
    }

    pub fn unfreeze(&mut self) -> Result<Idempotent<()>, CustomerError> {
        if self.is_closed() {
            return Err(CustomerError::CustomerIsClosed);
        }
        idempotency_guard!(
            self.events.iter_all().rev(),
            CustomerEvent::Unfrozen { .. },
            => CustomerEvent::Frozen { .. }
        );
        if !self.is_frozen() {
            return Ok(Idempotent::AlreadyApplied);
        }
        let status = CustomerStatus::Active;
        self.events.push(CustomerEvent::Unfrozen { status });
        self.status = status;
        Ok(Idempotent::Executed(()))
    }
}

impl TryFromEvents<CustomerEvent> for Customer {
    fn try_from_events(events: EntityEvents<CustomerEvent>) -> Result<Self, EntityHydrationError> {
        let mut builder = CustomerBuilder::default();

        for event in events.iter_all() {
            match event {
                CustomerEvent::Initialized {
                    id,
                    party_id,
                    customer_type,
                    public_id,
                    conversion,
                    level,
                } => {
                    builder = builder
                        .id(*id)
                        .party_id(*party_id)
                        .customer_type(*customer_type)
                        .status(CustomerStatus::Active)
                        .public_id(public_id.clone())
                        .level(*level)
                        .conversion(conversion.clone());
                }
                CustomerEvent::Frozen { status } => {
                    builder = builder.status(*status);
                }
                CustomerEvent::Unfrozen { status } => {
                    builder = builder.status(*status);
                }
                CustomerEvent::Closed { status } => {
                    builder = builder.status(*status);
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewCustomer {
    #[builder(setter(into))]
    pub(crate) id: CustomerId,
    pub(crate) party_id: PartyId,
    pub(crate) customer_type: CustomerType,
    #[builder(setter(into))]
    pub(crate) public_id: PublicId,
    pub(crate) conversion: CustomerConversion,
    pub(crate) level: KycLevel,
    #[builder(setter(skip), default)]
    pub(crate) status: CustomerStatus,
}

impl NewCustomer {
    pub fn builder() -> NewCustomerBuilder {
        NewCustomerBuilder::default()
    }
}

impl IntoEvents<CustomerEvent> for NewCustomer {
    fn into_events(self) -> EntityEvents<CustomerEvent> {
        EntityEvents::init(
            self.id,
            [CustomerEvent::Initialized {
                id: self.id,
                party_id: self.party_id,
                customer_type: self.customer_type,
                public_id: self.public_id,
                conversion: self.conversion,
                level: self.level,
            }],
        )
    }
}
