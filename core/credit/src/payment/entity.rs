use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::*;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct PaymentAccountIds {
    pub disbursed_receivable_account_id: CalaAccountId,
    pub interest_receivable_account_id: CalaAccountId,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PaymentId")]
pub enum PaymentEvent {
    Initialized {
        id: PaymentId,
        ledger_tx_id: LedgerTxId,
        ledger_tx_ref: String,
        obligation_id: ObligationId,
        amount: UsdCents,
        receivable_account_id: CalaAccountId,
        account_to_be_debited_id: CalaAccountId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Payment {
    pub id: PaymentId,
    pub ledger_tx_id: LedgerTxId,
    pub ledger_tx_ref: String,
    pub obligation_id: ObligationId,
    pub amount: UsdCents,
    pub receivable_account_id: CalaAccountId,
    pub account_to_be_debited_id: CalaAccountId,

    pub(super) events: EntityEvents<PaymentEvent>,
}

impl TryFromEvents<PaymentEvent> for Payment {
    fn try_from_events(events: EntityEvents<PaymentEvent>) -> Result<Self, EsEntityError> {
        let mut builder = PaymentBuilder::default();
        for event in events.iter_all() {
            match event {
                PaymentEvent::Initialized {
                    id,
                    ledger_tx_id,
                    ledger_tx_ref,
                    obligation_id,
                    receivable_account_id,
                    account_to_be_debited_id,
                    amount,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .ledger_tx_id(*ledger_tx_id)
                        .ledger_tx_ref(ledger_tx_ref.clone())
                        .obligation_id(*obligation_id)
                        .amount(*amount)
                        .receivable_account_id(*receivable_account_id)
                        .account_to_be_debited_id(*account_to_be_debited_id)
                }
            }
        }
        builder.events(events).build()
    }
}

impl Payment {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }
}

#[derive(Debug, Builder)]
pub struct NewPayment {
    #[builder(setter(into))]
    pub(super) id: PaymentId,
    #[builder(setter(into))]
    pub(super) ledger_tx_id: LedgerTxId,
    #[builder(setter(into))]
    pub(super) ledger_tx_ref: String,
    #[builder(setter(into))]
    pub(super) obligation_id: ObligationId,
    pub(super) amount: UsdCents,
    #[builder(setter(into))]
    pub(super) receivable_account_id: CalaAccountId,
    #[builder(setter(into))]
    pub(super) account_to_be_debited_id: CalaAccountId,
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
                ledger_tx_id: self.ledger_tx_id,
                ledger_tx_ref: self.ledger_tx_ref,
                obligation_id: self.obligation_id,
                amount: self.amount,
                receivable_account_id: self.receivable_account_id,
                account_to_be_debited_id: self.account_to_be_debited_id,
                audit_info: self.audit_info,
            }],
        )
    }
}
