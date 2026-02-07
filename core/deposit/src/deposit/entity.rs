use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;
use money::UsdCents;

use crate::primitives::{CalaTransactionId, DepositAccountId, DepositId, DepositStatus, PublicId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositId")]
pub enum DepositEvent {
    Initialized {
        id: DepositId,
        ledger_tx_id: CalaTransactionId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: String,
        status: DepositStatus,
        public_id: PublicId,
    },
    Reverted {
        ledger_tx_id: CalaTransactionId,
        status: DepositStatus,
    },
}

#[derive(Debug)]
pub struct DepositReversalData {
    pub entity_id: DepositId,
    pub ledger_tx_id: CalaTransactionId,
    pub credit_account_id: DepositAccountId,
    pub amount: UsdCents,
    pub correlation_id: String,
    pub external_id: String,
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Deposit {
    pub id: DepositId,
    pub deposit_account_id: DepositAccountId,
    pub amount: UsdCents,
    pub reference: String,
    pub public_id: PublicId,
    events: EntityEvents<DepositEvent>,
}

impl Deposit {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for deposit")
    }

    pub fn status(&self) -> DepositStatus {
        self.events
            .iter_all()
            .rev()
            .map(|event| match event {
                DepositEvent::Initialized { status, .. } => *status,
                DepositEvent::Reverted { status, .. } => *status,
            })
            .next()
            .expect("status should always exist")
    }

    pub fn revert(&mut self) -> Idempotent<DepositReversalData> {
        idempotency_guard!(
            self.events().iter_all().rev(),
            DepositEvent::Reverted { .. }
        );

        let ledger_tx_id = CalaTransactionId::new();
        self.events.push(DepositEvent::Reverted {
            ledger_tx_id,
            status: DepositStatus::Reverted,
        });

        Idempotent::Executed(DepositReversalData {
            entity_id: self.id,
            ledger_tx_id,
            credit_account_id: self.deposit_account_id,
            amount: self.amount,
            correlation_id: self.id.to_string(),
            external_id: format!("lana:deposit:{}:reverted", self.id),
        })
    }

    pub fn ledger_tx_ids(&self) -> Vec<CalaTransactionId> {
        self.events
            .iter_all()
            .map(|e| match e {
                DepositEvent::Initialized { ledger_tx_id, .. } => *ledger_tx_id,
                DepositEvent::Reverted { ledger_tx_id, .. } => *ledger_tx_id,
            })
            .collect()
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
                    public_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .deposit_account_id(*deposit_account_id)
                        .amount(*amount)
                        .reference(reference.clone())
                        .public_id(public_id.clone());
                }
                DepositEvent::Reverted { .. } => {}
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
    #[builder(setter(into))]
    pub(super) public_id: PublicId,
    reference: Option<String>,
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
                ledger_tx_id: self.ledger_transaction_id,
                deposit_account_id: self.deposit_account_id,
                amount: self.amount,
                status: DepositStatus::Confirmed,
                public_id: self.public_id,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn errors_when_zero_amount_deposit_amount_is_passed() {
        let deposit = NewDeposit::builder()
            .id(DepositId::new())
            .ledger_transaction_id(CalaTransactionId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ZERO)
            .reference(None)
            .public_id(PublicId::new("test-public-id"))
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
            .public_id(PublicId::new("test-public-id"))
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
            .public_id(PublicId::new("test-public-id"))
            .build();

        assert!(deposit.is_ok());
    }

    #[test]
    fn revert_deposit() {
        let new_deposit = NewDeposit::builder()
            .id(DepositId::new())
            .ledger_transaction_id(CalaTransactionId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ONE)
            .reference(None)
            .public_id(PublicId::new("test-public-id"))
            .build()
            .unwrap();

        let mut deposit = Deposit::try_from_events(new_deposit.into_events()).unwrap();
        assert_eq!(deposit.status(), DepositStatus::Confirmed);

        let res = deposit.revert();
        assert!(res.did_execute());

        let res = deposit.revert();
        assert!(res.was_already_applied());
    }
}
