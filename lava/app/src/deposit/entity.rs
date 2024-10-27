use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{
    audit::AuditInfo,
    primitives::{CustomerId, DepositId, LedgerAccountId, UsdCents},
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositId")]
pub enum DepositEvent {
    Initialized {
        id: DepositId,
        customer_id: CustomerId,
        amount: UsdCents,
        reference: String,
        credit_account_id: LedgerAccountId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Deposit {
    pub id: DepositId,
    pub customer_id: CustomerId,
    pub amount: UsdCents,
    pub reference: String,
    pub credit_account_id: LedgerAccountId,
    pub(super) events: EntityEvents<DepositEvent>,
}

impl std::fmt::Display for Deposit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Deposit {}, uid: {}", self.id, self.customer_id)
    }
}

impl Deposit {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for deposit")
    }
}

impl TryFromEvents<DepositEvent> for Deposit {
    fn try_from_events(events: EntityEvents<DepositEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DepositBuilder::default();
        for event in events.iter_all() {
            match event {
                DepositEvent::Initialized {
                    id,
                    customer_id,
                    amount,
                    credit_account_id,
                    reference,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .customer_id(*customer_id)
                        .amount(*amount)
                        .credit_account_id(*credit_account_id)
                        .reference(reference.clone());
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewDeposit {
    #[builder(setter(into))]
    pub(super) id: DepositId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    #[builder(setter(into))]
    pub(super) amount: UsdCents,
    reference: Option<String>,
    pub(super) credit_account_id: LedgerAccountId,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewDeposit {
    pub fn builder() -> NewDepositBuilder {
        NewDepositBuilder::default()
    }

    pub(super) fn reference(&self) -> String {
        match self.reference.as_deref() {
            None => self.id.to_string(),
            Some("") => self.id.to_string(),
            Some(reference) => reference.to_string(),
        }
    }
}

impl IntoEvents<DepositEvent> for NewDeposit {
    fn into_events(self) -> EntityEvents<DepositEvent> {
        EntityEvents::init(
            self.id,
            [DepositEvent::Initialized {
                reference: self.reference(),
                id: self.id,
                customer_id: self.customer_id,
                amount: self.amount,
                credit_account_id: self.credit_account_id,
                audit_info: self.audit_info,
            }],
        )
    }
}
