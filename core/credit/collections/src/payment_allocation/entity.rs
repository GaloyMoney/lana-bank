use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PaymentAllocationId")]
pub enum PaymentAllocationEvent {
    Initialized {
        id: PaymentAllocationId,
        ledger_tx_id: LedgerTxId,
        payment_id: PaymentId,
        obligation_id: ObligationId,
        payment_allocation_idx: usize,
        obligation_type: ObligationType,
        credit_facility_id: BeneficiaryId,
        amount: UsdCents,
        receivable_account_id: CalaAccountId,
        payment_holding_account_id: CalaAccountId,
        effective: chrono::NaiveDate,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct PaymentAllocation {
    pub id: PaymentAllocationId,
    pub obligation_id: ObligationId,
    pub payment_allocation_idx: usize,
    pub obligation_type: ObligationType,
    pub credit_facility_id: BeneficiaryId,
    pub ledger_tx_id: LedgerTxId,
    pub amount: UsdCents,
    pub payment_holding_account_id: CalaAccountId,
    pub receivable_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,

    events: EntityEvents<PaymentAllocationEvent>,
}

impl PaymentAllocation {
    pub fn tx_ref(&self) -> String {
        format!(
            "obligation-{}-idx-{}",
            self.obligation_id, self.payment_allocation_idx,
        )
    }
}

impl TryFromEvents<PaymentAllocationEvent> for PaymentAllocation {
    fn try_from_events(
        events: EntityEvents<PaymentAllocationEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = PaymentAllocationBuilder::default();
        for event in events.iter_all() {
            match event {
                PaymentAllocationEvent::Initialized {
                    id,
                    obligation_id,
                    payment_allocation_idx,
                    obligation_type,
                    credit_facility_id,
                    ledger_tx_id,
                    amount,
                    payment_holding_account_id,
                    receivable_account_id,
                    effective,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .obligation_id(*obligation_id)
                        .payment_allocation_idx(*payment_allocation_idx)
                        .obligation_type(*obligation_type)
                        .credit_facility_id(*credit_facility_id)
                        .ledger_tx_id(*ledger_tx_id)
                        .amount(*amount)
                        .payment_holding_account_id(*payment_holding_account_id)
                        .receivable_account_id(*receivable_account_id)
                        .effective(*effective)
                }
            }
        }
        builder.events(events).build()
    }
}

impl PaymentAllocation {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }
}

#[derive(Debug, Builder, Clone)]
pub struct NewPaymentAllocation {
    #[builder(setter(into))]
    pub(crate) id: PaymentAllocationId,
    pub(crate) payment_id: PaymentId,
    pub(crate) obligation_id: ObligationId,
    pub(crate) obligation_type: ObligationType,
    pub(crate) payment_allocation_idx: usize,
    pub(crate) credit_facility_id: BeneficiaryId,
    pub(crate) receivable_account_id: CalaAccountId,
    pub(crate) payment_holding_account_id: CalaAccountId,
    pub(crate) effective: chrono::NaiveDate,
    #[builder(setter(into))]
    pub(crate) amount: UsdCents,
}

impl NewPaymentAllocation {
    pub fn builder() -> NewPaymentAllocationBuilder {
        NewPaymentAllocationBuilder::default()
    }
}
impl IntoEvents<PaymentAllocationEvent> for NewPaymentAllocation {
    fn into_events(self) -> EntityEvents<PaymentAllocationEvent> {
        EntityEvents::init(
            self.id,
            [PaymentAllocationEvent::Initialized {
                id: self.id,
                ledger_tx_id: self.id.into(),
                payment_id: self.payment_id,
                obligation_id: self.obligation_id,
                payment_allocation_idx: self.payment_allocation_idx,
                obligation_type: self.obligation_type,
                credit_facility_id: self.credit_facility_id,
                amount: self.amount,
                payment_holding_account_id: self.payment_holding_account_id,
                effective: self.effective,
                receivable_account_id: self.receivable_account_id,
            }],
        )
    }
}
