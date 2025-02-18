use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{audit::AuditInfo, credit_facility::*, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PaymentId")]
pub enum PaymentEvent {
    Initialized {
        id: PaymentId,
        ledger_tx_id: LedgerTxId,
        ledger_tx_ref: String,
        facility_id: CreditFacilityId,
        amounts: CreditFacilityPaymentAmounts,
        account_ids: CreditFacilityAccountIds,
        deposit_account_id: DepositAccountId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Payment {
    pub id: PaymentId,
    pub ledger_tx_id: LedgerTxId,
    pub ledger_tx_ref: String,
    pub facility_id: CreditFacilityId,
    pub amounts: CreditFacilityPaymentAmounts,
    pub account_ids: CreditFacilityAccountIds,
    pub deposit_account_id: DepositAccountId,

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
                    facility_id,
                    account_ids,
                    amounts,
                    deposit_account_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .ledger_tx_id(*ledger_tx_id)
                        .ledger_tx_ref(ledger_tx_ref.clone())
                        .facility_id(*facility_id)
                        .amounts(*amounts)
                        .account_ids(*account_ids)
                        .deposit_account_id(*deposit_account_id)
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
    pub(super) credit_facility_id: CreditFacilityId,
    pub(super) amounts: CreditFacilityPaymentAmounts,
    pub(super) account_ids: CreditFacilityAccountIds,
    pub(super) deposit_account_id: DepositAccountId,
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
                facility_id: self.credit_facility_id,
                amounts: self.amounts,
                account_ids: self.account_ids,
                deposit_account_id: self.deposit_account_id,
                audit_info: self.audit_info,
            }],
        )
    }
}
