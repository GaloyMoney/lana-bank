use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use core_money::UsdCents;
use es_entity::*;

use crate::primitives::{CalaTransactionId, DepositAccountId, DepositId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
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
    events: EntityEvents<DepositEvent>,
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
#[builder(build_fn(validate = "Self::validate"))]
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

impl NewDepositBuilder {
    fn validate(&self) -> Result<(), String> {
        match self.amount {
            Some(amount) if amount.is_zero() => Err("Deposit amount cannot be zero".to_string()),
            _ => Ok(()),
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

    #[test]
    fn errors_when_zero_amount_deposit_amount_is_passed() {
        let deposit = NewDeposit::builder()
            .id(DepositId::new())
            .ledger_transaction_id(CalaTransactionId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ZERO)
            .reference(None)
            .audit_info(dummy_audit_info())
            .build();

        assert!(matches!(
            deposit,
            Err(NewDepositBuilderError::ValidationError(_))
        ));
    }

    #[test]
    fn errors_when_amount_is_not_provided() {
        let deposit = NewDeposit::builder()
            .id(DepositId::new())
            .ledger_transaction_id(CalaTransactionId::new())
            .deposit_account_id(DepositAccountId::new())
            .reference(None)
            .audit_info(dummy_audit_info())
            .build();

        assert!(matches!(
            deposit,
            Err(NewDepositBuilderError::UninitializedField(_))
        ));
    }

    #[test]
    fn passes_when_all_inputs_provided() {
        let deposit = NewDeposit::builder()
            .id(DepositId::new())
            .ledger_transaction_id(CalaTransactionId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ONE)
            .reference(None)
            .audit_info(dummy_audit_info())
            .build();

        assert!(deposit.is_ok());
    }
}
