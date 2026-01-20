use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

use super::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PaymentId")]
pub enum PaymentEvent {
    Initialized {
        id: PaymentId,
        ledger_tx_id: LedgerTxId,
        credit_facility_id: CreditFacilityId,
        facility_payment_holding_account_id: CalaAccountId,
        facility_uncovered_outstanding_account_id: CalaAccountId,
        payment_source_account_id: CalaAccountId,
        amount: UsdCents,
        effective: chrono::NaiveDate,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Payment {
    pub id: PaymentId,
    pub ledger_tx_id: LedgerTxId,
    pub credit_facility_id: CreditFacilityId,
    pub facility_payment_holding_account_id: CalaAccountId,
    pub facility_uncovered_outstanding_account_id: CalaAccountId,
    pub payment_source_account_id: CalaAccountId,
    pub amount: UsdCents,
    pub effective: chrono::NaiveDate,

    events: EntityEvents<PaymentEvent>,
}

impl TryFromEvents<PaymentEvent> for Payment {
    fn try_from_events(events: EntityEvents<PaymentEvent>) -> Result<Self, EsEntityError> {
        let mut builder = PaymentBuilder::default();
        for event in events.iter_all() {
            match event {
                PaymentEvent::Initialized {
                    id,
                    ledger_tx_id,
                    credit_facility_id,
                    facility_payment_holding_account_id,
                    facility_uncovered_outstanding_account_id,
                    payment_source_account_id,
                    amount,
                    effective,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .ledger_tx_id(*ledger_tx_id)
                        .credit_facility_id(*credit_facility_id)
                        .facility_payment_holding_account_id(*facility_payment_holding_account_id)
                        .facility_uncovered_outstanding_account_id(
                            *facility_uncovered_outstanding_account_id,
                        )
                        .payment_source_account_id(*payment_source_account_id)
                        .amount(*amount)
                        .effective(*effective)
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

    pub(crate) fn tx_ref(&self) -> String {
        format!(
            "credit-facility-{}-idx-{}",
            self.credit_facility_id, self.id,
        )
    }
}

impl From<Payment> for PaymentDetailsForAllocation {
    fn from(payment: Payment) -> Self {
        Self {
            payment_id: payment.id,
            amount: payment.amount,
            credit_facility_id: payment.credit_facility_id,
            facility_payment_holding_account_id: payment.facility_payment_holding_account_id,
            effective: payment.effective,
        }
    }
}

#[derive(Debug, Builder)]
pub struct NewPayment {
    #[builder(setter(into))]
    pub(super) id: PaymentId,
    #[builder(setter(into))]
    pub(super) ledger_tx_id: LedgerTxId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    pub(super) payment_ledger_account_ids: PaymentLedgerAccountIds,
    pub(super) amount: UsdCents,
    pub(crate) effective: chrono::NaiveDate,
}

impl NewPayment {
    pub fn builder() -> NewPaymentBuilder {
        NewPaymentBuilder::default()
    }
}
impl IntoEvents<PaymentEvent> for NewPayment {
    fn into_events(self) -> EntityEvents<PaymentEvent> {
        let PaymentLedgerAccountIds {
            facility_payment_holding_account_id,
            facility_uncovered_outstanding_account_id,
            payment_source_account_id,
        } = self.payment_ledger_account_ids;
        EntityEvents::init(
            self.id,
            [PaymentEvent::Initialized {
                id: self.id,
                ledger_tx_id: self.ledger_tx_id,
                credit_facility_id: self.credit_facility_id,
                facility_payment_holding_account_id,
                facility_uncovered_outstanding_account_id,
                payment_source_account_id: payment_source_account_id.into(),
                amount: self.amount,
                effective: self.effective,
            }],
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PaymentLedgerAccountIds {
    pub(crate) facility_payment_holding_account_id: CalaAccountId,
    pub(crate) facility_uncovered_outstanding_account_id: CalaAccountId,
    pub(crate) payment_source_account_id: PaymentSourceAccountId,
}
