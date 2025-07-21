use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::*;

pub struct AllocatedAmounts {
    pub disbursal: UsdCents,
    pub interest: UsdCents,
}

impl Default for AllocatedAmounts {
    fn default() -> Self {
        Self {
            disbursal: UsdCents::ZERO,
            interest: UsdCents::ZERO,
        }
    }
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PaymentId")]
pub enum PaymentEvent {
    Initialized {
        id: PaymentId,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
        effective: chrono::NaiveDate,
        audit_info: AuditInfo,
    },
    PaymentAllocated {
        disbursal: UsdCents,
        interest: UsdCents,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Payment {
    pub id: PaymentId,
    pub credit_facility_id: CreditFacilityId,
    pub initial_amount: UsdCents,
    pub applied_amount: UsdCents,
    pub effective: chrono::NaiveDate,

    events: EntityEvents<PaymentEvent>,
}

impl TryFromEvents<PaymentEvent> for Payment {
    fn try_from_events(events: EntityEvents<PaymentEvent>) -> Result<Self, EsEntityError> {
        let mut builder = PaymentBuilder::default();
        let mut applied_amount = UsdCents::ZERO;
        for event in events.iter_all() {
            match event {
                PaymentEvent::Initialized {
                    id,
                    credit_facility_id,
                    amount,
                    effective,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .credit_facility_id(*credit_facility_id)
                        .initial_amount(*amount)
                        .effective(*effective)
                }
                PaymentEvent::PaymentAllocated {
                    disbursal,
                    interest,
                    ..
                } => applied_amount += *disbursal + *interest,
            }
        }
        builder
            .applied_amount(applied_amount)
            .events(events)
            .build()
    }
}

impl Payment {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn allocated_amounts(&self) -> AllocatedAmounts {
        self.events
            .iter_all()
            .find_map(|event| match event {
                PaymentEvent::PaymentAllocated {
                    disbursal,
                    interest,
                    ..
                } => Some(AllocatedAmounts {
                    disbursal: *disbursal,
                    interest: *interest,
                }),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn record_allocated(
        &mut self,
        disbursal: UsdCents,
        interest: UsdCents,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            PaymentEvent::PaymentAllocated { .. }
        );

        self.events.push(PaymentEvent::PaymentAllocated {
            disbursal,
            interest,
            audit_info,
        });

        self.applied_amount += disbursal + interest;

        Idempotent::Executed(())
    }
}

#[derive(Debug, Builder)]
pub struct NewPayment {
    #[builder(setter(into))]
    pub(super) id: PaymentId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    pub(super) amount: UsdCents,
    effective: chrono::NaiveDate,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
}

impl NewPayment {
    pub fn builder() -> NewPaymentBuilder {
        NewPaymentBuilder::default()
    }
}
impl IntoEvents<PaymentEvent> for NewPayment {
    fn into_events(self) -> EntityEvents<PaymentEvent> {
        EntityEvents::init(
            self.id,
            [PaymentEvent::Initialized {
                id: self.id,
                credit_facility_id: self.credit_facility_id,
                amount: self.amount,
                effective: self.effective,
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use audit::AuditEntryId;

    use super::*;

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn accrual_from(events: Vec<PaymentEvent>) -> Payment {
        let id = events
            .iter()
            .find_map(|event| match event {
                PaymentEvent::Initialized { id, .. } => Some(*id),
                _ => None,
            })
            .expect("Initialized event not found");
        Payment::try_from_events(EntityEvents::init(id, events)).unwrap()
    }

    #[test]
    fn can_instantiate_payment_entity() {
        let id = PaymentId::new();
        let events = vec![{
            PaymentEvent::Initialized {
                id,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::ONE,
                effective: Utc::now().date_naive(),
                audit_info: dummy_audit_info(),
            }
        }];
        let payment = accrual_from(events);
        assert_eq!(payment.id, id);
    }
}
