use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::{
    ApprovalProcessId, CalaTransactionId, DepositAccountId, PublicId, UsdCents, WithdrawalId,
};

use super::error::WithdrawalError;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum WithdrawalStatus {
    PendingApproval,
    PendingConfirmation,
    Confirmed,
    Denied,
    Cancelled,
    Reverted,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "WithdrawalId")]
pub enum WithdrawalEvent {
    Initialized {
        id: WithdrawalId,
        ledger_tx_id: CalaTransactionId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: String,
        approval_process_id: ApprovalProcessId,
        status: WithdrawalStatus,
        public_id: PublicId,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        approved: bool,
        status: WithdrawalStatus,
    },
    Denied {
        ledger_tx_id: CalaTransactionId,
        status: WithdrawalStatus,
    },
    Confirmed {
        ledger_tx_id: CalaTransactionId,
        status: WithdrawalStatus,
    },
    Cancelled {
        ledger_tx_id: CalaTransactionId,
        status: WithdrawalStatus,
    },
    Reverted {
        ledger_tx_id: CalaTransactionId,
        status: WithdrawalStatus,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Withdrawal {
    pub id: WithdrawalId,
    pub deposit_account_id: DepositAccountId,
    pub reference: String,
    pub amount: UsdCents,
    pub approval_process_id: ApprovalProcessId,
    pub public_id: PublicId,
    #[builder(setter(strip_option), default)]
    pub cancelled_tx_id: Option<CalaTransactionId>,

    events: EntityEvents<WithdrawalEvent>,
}

#[derive(Debug)]
pub struct WithdrawalReversalData {
    pub entity_id: WithdrawalId,
    pub ledger_tx_id: CalaTransactionId,
    pub credit_account_id: DepositAccountId,
    pub amount: UsdCents,
    pub correlation_id: String,
    pub external_id: String,
}

impl Withdrawal {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for deposit")
    }

    pub fn confirm(&mut self) -> Result<Idempotent<CalaTransactionId>, WithdrawalError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            WithdrawalEvent::Confirmed { .. }
        );

        match self.is_approved_or_denied() {
            Some(false) => return Err(WithdrawalError::NotApproved(self.id)),
            None => return Err(WithdrawalError::NotApproved(self.id)),
            _ => (),
        }

        if self.is_cancelled() {
            return Err(WithdrawalError::AlreadyCancelled(self.id));
        }

        let ledger_tx_id = CalaTransactionId::new();
        self.events.push(WithdrawalEvent::Confirmed {
            ledger_tx_id,
            status: WithdrawalStatus::Confirmed,
        });

        Ok(Idempotent::Executed(ledger_tx_id))
    }

    pub fn revert(&mut self) -> Result<Idempotent<WithdrawalReversalData>, WithdrawalError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            WithdrawalEvent::Reverted { .. }
        );

        if self.is_cancelled() {
            return Err(WithdrawalError::AlreadyCancelled(self.id));
        }

        if !self.is_confirmed() {
            return Err(WithdrawalError::NotConfirmed(self.id));
        }

        let ledger_tx_id = CalaTransactionId::new();

        self.events.push(WithdrawalEvent::Reverted {
            ledger_tx_id,
            status: WithdrawalStatus::Reverted,
        });

        Ok(Idempotent::Executed(WithdrawalReversalData {
            entity_id: self.id,
            ledger_tx_id,
            amount: self.amount,
            credit_account_id: self.deposit_account_id,
            correlation_id: self.id.to_string(),
            external_id: format!("lana:withdraw:{}:reverted", self.id),
        }))
    }

    pub fn cancel(&mut self) -> Result<Idempotent<CalaTransactionId>, WithdrawalError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            WithdrawalEvent::Cancelled { .. }
        );

        if self.is_confirmed() {
            return Err(WithdrawalError::AlreadyConfirmed(self.id));
        }

        let ledger_tx_id = CalaTransactionId::new();
        self.events.push(WithdrawalEvent::Cancelled {
            ledger_tx_id,
            status: WithdrawalStatus::Cancelled,
        });
        self.cancelled_tx_id = Some(ledger_tx_id);

        Ok(Idempotent::Executed(ledger_tx_id))
    }

    fn is_confirmed(&self) -> bool {
        self.events
            .iter_all()
            .any(|e| matches!(e, WithdrawalEvent::Confirmed { .. }))
    }

    pub fn is_approved_or_denied(&self) -> Option<bool> {
        self.events.iter_all().find_map(|e| {
            if let WithdrawalEvent::ApprovalProcessConcluded { approved, .. } = e {
                Some(*approved)
            } else {
                None
            }
        })
    }

    fn is_cancelled(&self) -> bool {
        self.events
            .iter_all()
            .rev()
            .any(|e| matches!(e, WithdrawalEvent::Cancelled { .. }))
    }

    pub fn status(&self) -> WithdrawalStatus {
        self.events
            .iter_all()
            .rev()
            .map(|e| match e {
                WithdrawalEvent::Confirmed { status, .. } => *status,
                WithdrawalEvent::Cancelled { status, .. } => *status,
                WithdrawalEvent::Reverted { status, .. } => *status,
                WithdrawalEvent::Denied { status, .. } => *status,
                WithdrawalEvent::ApprovalProcessConcluded { status, .. } => *status,
                WithdrawalEvent::Initialized { status, .. } => *status,
            })
            .next()
            .expect("status should always exist")
    }

    pub fn ledger_tx_ids(&self) -> Vec<CalaTransactionId> {
        self.events
            .iter_all()
            .filter_map(|e| match e {
                WithdrawalEvent::Initialized { ledger_tx_id, .. } => Some(*ledger_tx_id),
                WithdrawalEvent::Denied { ledger_tx_id, .. } => Some(*ledger_tx_id),
                WithdrawalEvent::Confirmed { ledger_tx_id, .. } => Some(*ledger_tx_id),
                WithdrawalEvent::Cancelled { ledger_tx_id, .. } => Some(*ledger_tx_id),
                WithdrawalEvent::Reverted { ledger_tx_id, .. } => Some(*ledger_tx_id),
                WithdrawalEvent::ApprovalProcessConcluded { .. } => None,
            })
            .collect()
    }

    pub fn approval_process_concluded(
        &mut self,
        approved: bool,
    ) -> Idempotent<Option<CalaTransactionId>> {
        idempotency_guard!(
            self.events.iter_all(),
            WithdrawalEvent::ApprovalProcessConcluded { .. }
        );
        let status = if approved {
            WithdrawalStatus::PendingConfirmation
        } else {
            WithdrawalStatus::Denied
        };
        self.events.push(WithdrawalEvent::ApprovalProcessConcluded {
            approval_process_id: self.id.into(),
            approved,
            status,
        });

        if approved {
            return Idempotent::Executed(None);
        }

        let ledger_tx_id = CalaTransactionId::new();
        self.events.push(WithdrawalEvent::Denied {
            ledger_tx_id,
            status,
        });

        Idempotent::Executed(Some(ledger_tx_id))
    }
}

impl TryFromEvents<WithdrawalEvent> for Withdrawal {
    fn try_from_events(events: EntityEvents<WithdrawalEvent>) -> Result<Self, EsEntityError> {
        let mut builder = WithdrawalBuilder::default();
        for event in events.iter_all() {
            match event {
                WithdrawalEvent::Initialized {
                    id,
                    reference,
                    deposit_account_id,
                    amount,
                    approval_process_id,
                    public_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .deposit_account_id(*deposit_account_id)
                        .amount(*amount)
                        .reference(reference.clone())
                        .approval_process_id(*approval_process_id)
                        .public_id(public_id.clone())
                }
                WithdrawalEvent::Cancelled { ledger_tx_id, .. } => {
                    builder = builder.cancelled_tx_id(*ledger_tx_id)
                }
                _ => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct NewWithdrawal {
    #[builder(setter(into))]
    pub(super) id: WithdrawalId,
    #[builder(setter(into))]
    pub(super) deposit_account_id: DepositAccountId,
    #[builder(setter(into))]
    pub(super) amount: UsdCents,
    #[builder(setter(into))]
    pub(super) approval_process_id: ApprovalProcessId,
    #[builder(setter(into))]
    pub(super) public_id: PublicId,
    reference: Option<String>,
}

impl NewWithdrawal {
    pub fn builder() -> NewWithdrawalBuilder {
        NewWithdrawalBuilder::default()
    }

    pub(super) fn reference(&self) -> String {
        match self.reference.as_deref() {
            None => self.id.to_string(),
            Some("") => self.id.to_string(),
            Some(reference) => reference.to_string(),
        }
    }
}

impl NewWithdrawalBuilder {
    fn validate(&self) -> Result<(), String> {
        match self.amount {
            Some(amount) if amount.is_zero() => Err("Withdrawal amount cannot be zero".to_string()),
            _ => Ok(()),
        }
    }
}

impl IntoEvents<WithdrawalEvent> for NewWithdrawal {
    fn into_events(self) -> EntityEvents<WithdrawalEvent> {
        EntityEvents::init(
            self.id,
            [WithdrawalEvent::Initialized {
                reference: self.reference(),
                id: self.id,
                ledger_tx_id: self.id.into(),
                deposit_account_id: self.deposit_account_id,
                amount: self.amount,
                approval_process_id: self.approval_process_id,
                status: WithdrawalStatus::PendingApproval,
                public_id: self.public_id,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn errors_when_zero_amount_withdrawal_amount_is_passed() {
        let withdrawal = NewWithdrawal::builder()
            .id(WithdrawalId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ZERO)
            .reference(None)
            .approval_process_id(ApprovalProcessId::new())
            .public_id(PublicId::new("test-public-id"))
            .build();

        assert!(matches!(
            withdrawal,
            Err(NewWithdrawalBuilderError::ValidationError(_))
        ));
    }

    #[test]
    fn errors_when_amount_is_not_provided() {
        let withdrawal = NewWithdrawal::builder()
            .id(WithdrawalId::new())
            .deposit_account_id(DepositAccountId::new())
            .reference(None)
            .approval_process_id(ApprovalProcessId::new())
            .public_id(PublicId::new("test-public-id"))
            .build();

        assert!(matches!(
            withdrawal,
            Err(NewWithdrawalBuilderError::UninitializedField(_))
        ));
    }

    #[test]
    fn passes_when_all_inputs_provided() {
        let withdrawal = NewWithdrawal::builder()
            .id(WithdrawalId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ONE)
            .reference(None)
            .approval_process_id(ApprovalProcessId::new())
            .public_id(PublicId::new("test-public-id"))
            .build();

        assert!(withdrawal.is_ok());
    }

    fn create_confirmed_withdrawal() -> Withdrawal {
        let new_withdrawal = NewWithdrawal::builder()
            .id(WithdrawalId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ONE)
            .reference(None)
            .approval_process_id(ApprovalProcessId::new())
            .public_id(PublicId::new("test-public-id"))
            .build()
            .unwrap();

        let mut withdrawal = Withdrawal::try_from_events(new_withdrawal.into_events()).unwrap();
        withdrawal.approval_process_concluded(true).unwrap();
        let _ = withdrawal.confirm().unwrap();
        withdrawal
    }

    #[test]
    fn can_revert_confirmed_withdrawal() {
        let mut withdrawal = create_confirmed_withdrawal();

        let result = withdrawal.revert();

        assert!(result.is_ok());
        assert_eq!(withdrawal.status(), WithdrawalStatus::Reverted);
    }

    #[test]
    fn cancelled_withdrawal_errors_on_revert() {
        let new_withdrawal = NewWithdrawal::builder()
            .id(WithdrawalId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ONE)
            .reference(None)
            .approval_process_id(ApprovalProcessId::new())
            .public_id(PublicId::new("test-public-id"))
            .build()
            .unwrap();

        let mut withdrawal = Withdrawal::try_from_events(new_withdrawal.into_events()).unwrap();
        withdrawal.approval_process_concluded(true).unwrap();
        let _ = withdrawal.cancel().unwrap();

        let result = withdrawal.revert();
        assert!(matches!(result, Err(WithdrawalError::AlreadyCancelled(_))));
    }

    #[test]
    fn reverted_withdrawal_is_ignored_on_revert() {
        let mut withdrawal = create_confirmed_withdrawal();

        let _ = withdrawal.revert().unwrap();
        let result = withdrawal.revert().unwrap();
        assert!(result.was_already_applied());
    }

    #[test]
    fn cannot_revert_unconfirmed_withdrawal() {
        let new_withdrawal = NewWithdrawal::builder()
            .id(WithdrawalId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ONE)
            .reference(None)
            .approval_process_id(ApprovalProcessId::new())
            .public_id(PublicId::new("test-public-id"))
            .build()
            .unwrap();

        let mut withdrawal = Withdrawal::try_from_events(new_withdrawal.into_events()).unwrap();
        withdrawal.approval_process_concluded(true).unwrap();

        let result = withdrawal.revert();

        assert!(matches!(result, Err(WithdrawalError::NotConfirmed(_))));
    }
}
