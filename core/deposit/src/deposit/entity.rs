use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use core_money::UsdCents;
use es_entity::*;

use crate::primitives::{CalaTransactionId, DepositAccountId, DepositId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositId")]
pub enum DepositEvent {
    Initialized {
        id: DepositId,
        ledger_transaction_id: CalaTransactionId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Deposit {
    pub id: DepositId,
    pub deposit_account_id: DepositAccountId,
    pub amount: UsdCents,
    pub reference: String,
    pub(super) events: EntityEvents<DepositEvent>,
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
                    reference,
                    deposit_account_id,
                    amount,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .deposit_account_id(*deposit_account_id)
                        .amount(*amount)
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
    pub(super) ledger_transaction_id: CalaTransactionId,
    #[builder(setter(into))]
    pub(super) deposit_account_id: DepositAccountId,
    #[builder(setter(into))]
    pub(super) amount: UsdCents,
    reference: Option<String>,
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
                ledger_transaction_id: self.ledger_transaction_id,
                deposit_account_id: self.deposit_account_id,
                amount: self.amount,
                audit_info: self.audit_info,
            }],
        )
    }
}
