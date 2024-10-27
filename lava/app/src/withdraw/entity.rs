use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use super::error::*;
use crate::{
    audit::AuditInfo,
    primitives::{CustomerId, LedgerAccountId, LedgerTxId, UsdCents, WithdrawId},
};

#[derive(async_graphql::Enum, Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum WithdrawalStatus {
    Initiated,
    Cancelled,
    Confirmed,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "WithdrawId")]
pub enum WithdrawEvent {
    Initialized {
        id: WithdrawId,
        customer_id: CustomerId,
        amount: UsdCents,
        reference: String,
        debit_account_id: LedgerAccountId,
        ledger_tx_id: LedgerTxId,
        audit_info: AuditInfo,
    },
    Confirmed {
        ledger_tx_id: LedgerTxId,
        audit_info: AuditInfo,
    },
    Cancelled {
        ledger_tx_id: LedgerTxId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Withdraw {
    pub id: WithdrawId,
    pub reference: String,
    pub customer_id: CustomerId,
    pub amount: UsdCents,
    pub debit_account_id: LedgerAccountId,
    pub(super) events: EntityEvents<WithdrawEvent>,
}

impl Withdraw {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("Withdraw has never been persisted")
    }

    pub(super) fn confirm(&mut self, audit_info: AuditInfo) -> Result<LedgerTxId, WithdrawError> {
        if self.is_confirmed() {
            return Err(WithdrawError::AlreadyConfirmed(self.id));
        }

        if self.is_cancelled() {
            return Err(WithdrawError::AlreadyCancelled(self.id));
        }

        let ledger_tx_id = LedgerTxId::new();
        self.events.push(WithdrawEvent::Confirmed {
            ledger_tx_id,
            audit_info,
        });

        Ok(ledger_tx_id)
    }

    pub(super) fn cancel(&mut self, audit_info: AuditInfo) -> Result<LedgerTxId, WithdrawError> {
        if self.is_confirmed() {
            return Err(WithdrawError::AlreadyConfirmed(self.id));
        }

        if self.is_cancelled() {
            return Err(WithdrawError::AlreadyCancelled(self.id));
        }

        let ledger_tx_id = LedgerTxId::new();
        self.events.push(WithdrawEvent::Cancelled {
            ledger_tx_id,
            audit_info,
        });
        Ok(ledger_tx_id)
    }

    fn is_confirmed(&self) -> bool {
        self.events
            .iter_all()
            .any(|e| matches!(e, WithdrawEvent::Confirmed { .. }))
    }

    fn is_cancelled(&self) -> bool {
        self.events
            .iter_all()
            .any(|e| matches!(e, WithdrawEvent::Cancelled { .. }))
    }

    pub fn status(&self) -> WithdrawalStatus {
        if self.is_confirmed() {
            WithdrawalStatus::Confirmed
        } else if self.is_cancelled() {
            WithdrawalStatus::Cancelled
        } else {
            WithdrawalStatus::Initiated
        }
    }
}

impl std::fmt::Display for Withdraw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Withdraw {}, uid: {}", self.id, self.customer_id)
    }
}

impl TryFromEvents<WithdrawEvent> for Withdraw {
    fn try_from_events(events: EntityEvents<WithdrawEvent>) -> Result<Self, EsEntityError> {
        let mut builder = WithdrawBuilder::default();
        for event in events.iter_all() {
            match event {
                WithdrawEvent::Initialized {
                    id,
                    customer_id,
                    amount,
                    debit_account_id,
                    reference,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .customer_id(*customer_id)
                        .amount(*amount)
                        .debit_account_id(*debit_account_id)
                        .reference(reference.clone())
                }
                WithdrawEvent::Confirmed { .. } => {}
                WithdrawEvent::Cancelled { .. } => {}
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewWithdraw {
    #[builder(setter(into))]
    pub(super) id: WithdrawId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    #[builder(setter(into))]
    pub(super) amount: UsdCents,
    pub(super) reference: Option<String>,
    pub(super) debit_account_id: LedgerAccountId,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
}

impl NewWithdraw {
    pub fn builder() -> NewWithdrawBuilder {
        NewWithdrawBuilder::default()
    }

    pub(super) fn reference(&self) -> String {
        match self.reference.as_deref() {
            None => self.id.to_string(),
            Some("") => self.id.to_string(),
            Some(reference) => reference.to_string(),
        }
    }
}

impl IntoEvents<WithdrawEvent> for NewWithdraw {
    fn into_events(self) -> EntityEvents<WithdrawEvent> {
        EntityEvents::init(
            self.id,
            [WithdrawEvent::Initialized {
                reference: self.reference(),
                id: self.id,
                ledger_tx_id: LedgerTxId::from(uuid::Uuid::from(self.id)),
                customer_id: self.customer_id,
                amount: self.amount,
                debit_account_id: self.debit_account_id,
                audit_info: self.audit_info,
            }],
        )
    }
}
