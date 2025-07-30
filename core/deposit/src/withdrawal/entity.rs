use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::{
    ApprovalProcessId, CalaTransactionId, DepositAccountId, UsdCents, WithdrawalId,
};
use audit::AuditInfo;

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
        audit_info: AuditInfo,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        approved: bool,
        audit_info: AuditInfo,
    },
    Confirmed {
        ledger_tx_id: CalaTransactionId,
        audit_info: AuditInfo,
    },
    Cancelled {
        ledger_tx_id: CalaTransactionId,
        audit_info: AuditInfo,
    },
    Reverted {
        ledger_tx_id: CalaTransactionId,
        audit_info: AuditInfo,
    },
    StatusUpdated {
        status: WithdrawalStatus,
        audit_info: AuditInfo,
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
    #[builder(setter(strip_option), default)]
    pub cancelled_tx_id: Option<CalaTransactionId>,

    events: EntityEvents<WithdrawalEvent>,
}

#[derive(Debug)]
pub struct WithdrawalReversalData {
    pub ledger_tx_id: CalaTransactionId,
    pub credit_account_id: DepositAccountId,
    pub amount: UsdCents,
    pub correlation_id: String,
    pub external_id: String,
}

#[derive(Debug)]
pub struct WithdrawalConfirmationData {
    pub ledger_tx_id: CalaTransactionId,
    pub correlation_id: String,
    pub amount: UsdCents,
    pub credit_account_id: DepositAccountId,
    pub external_id: String,
}

#[derive(Debug)]
pub struct WithdrawalCancellationData {
    pub ledger_tx_id: CalaTransactionId,
    pub amount: UsdCents,
    pub credit_account_id: DepositAccountId,
}

impl Withdrawal {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for deposit")
    }

    pub fn confirm(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<Idempotent<WithdrawalConfirmationData>, WithdrawalError> {
        idempotency_guard!(
            self.events.iter_all(),
            WithdrawalEvent::Confirmed { .. }
                | WithdrawalEvent::StatusUpdated {
                    status: WithdrawalStatus::Confirmed,
                    ..
                }
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
            audit_info: audit_info.clone(),
        });

        self.update_status(WithdrawalStatus::Confirmed, audit_info)
            .expect("status should update");

        Ok(Idempotent::Executed(WithdrawalConfirmationData {
            ledger_tx_id,
            correlation_id: self.id.to_string(),
            amount: self.amount,
            credit_account_id: self.deposit_account_id,
            external_id: format!("lana:withdraw:{}:confirm", self.id),
        }))
    }

    pub fn revert(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<Idempotent<WithdrawalReversalData>, WithdrawalError> {
        idempotency_guard!(
            self.events.iter_all(),
            WithdrawalEvent::Reverted { .. }
                | WithdrawalEvent::Cancelled { .. }
                | WithdrawalEvent::StatusUpdated {
                    status: WithdrawalStatus::Reverted,
                    ..
                }
        );

        if !self.is_confirmed() {
            return Err(WithdrawalError::NotConfirmed(self.id));
        }

        let ledger_tx_id = CalaTransactionId::new();

        self.events.push(WithdrawalEvent::Reverted {
            ledger_tx_id,
            audit_info: audit_info.clone(),
        });

        self.update_status(WithdrawalStatus::Reverted, audit_info)
            .expect("status should update");

        Ok(Idempotent::Executed(WithdrawalReversalData {
            ledger_tx_id,
            amount: self.amount,
            credit_account_id: self.deposit_account_id,
            correlation_id: self.id.to_string(),
            external_id: format!("lana:withdraw:{}:reverted", self.id),
        }))
    }

    pub fn cancel(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<Idempotent<WithdrawalCancellationData>, WithdrawalError> {
        idempotency_guard!(
            self.events.iter_all(),
            WithdrawalEvent::Cancelled { .. }
                | WithdrawalEvent::StatusUpdated {
                    status: WithdrawalStatus::Cancelled,
                    ..
                }
        );
        if self.is_confirmed() {
            return Err(WithdrawalError::AlreadyConfirmed(self.id));
        }

        let ledger_tx_id = CalaTransactionId::new();
        self.events.push(WithdrawalEvent::Cancelled {
            ledger_tx_id,
            audit_info: audit_info.clone(),
        });
        self.cancelled_tx_id = Some(ledger_tx_id);
        self.update_status(WithdrawalStatus::Cancelled, audit_info)
            .expect("status should update");

        Ok(Idempotent::Executed(WithdrawalCancellationData {
            ledger_tx_id,
            amount: self.amount,
            credit_account_id: self.deposit_account_id,
        }))
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
            .find_map(|e| match e {
                WithdrawalEvent::StatusUpdated { status, .. } => Some(*status),
                WithdrawalEvent::Initialized { status, .. } => Some(*status),
                _ => None,
            })
            .expect("status should always exist")
    }

    pub fn approval_process_concluded(
        &mut self,
        approved: bool,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            WithdrawalEvent::ApprovalProcessConcluded { .. }
                | WithdrawalEvent::StatusUpdated {
                    status: WithdrawalStatus::PendingConfirmation,
                    ..
                }
                | WithdrawalEvent::StatusUpdated {
                    status: WithdrawalStatus::Denied,
                    ..
                }
        );

        self.events.push(WithdrawalEvent::ApprovalProcessConcluded {
            approval_process_id: self.id.into(),
            approved,
            audit_info: audit_info.clone(),
        });
        let status = if approved {
            WithdrawalStatus::PendingConfirmation
        } else {
            WithdrawalStatus::Denied
        };

        self.update_status(status, audit_info)
            .expect("status should update");
        Idempotent::Executed(())
    }

    fn update_status(&mut self, status: WithdrawalStatus, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(
            self.events().iter_all().rev(),
            WithdrawalEvent::StatusUpdated { status: existing_status, ..  } if existing_status == &status,
            => WithdrawalEvent::StatusUpdated { .. }
        );

        self.events
            .push(WithdrawalEvent::StatusUpdated { status, audit_info });

        Idempotent::Executed(())
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
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .deposit_account_id(*deposit_account_id)
                        .amount(*amount)
                        .reference(reference.clone())
                        .approval_process_id(*approval_process_id)
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
    reference: Option<String>,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
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
    fn errors_when_zero_amount_withdrawal_amount_is_passed() {
        let withdrawal = NewWithdrawal::builder()
            .id(WithdrawalId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ZERO)
            .reference(None)
            .approval_process_id(ApprovalProcessId::new())
            .audit_info(dummy_audit_info())
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
            .audit_info(dummy_audit_info())
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
            .audit_info(dummy_audit_info())
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
            .audit_info(dummy_audit_info())
            .build()
            .unwrap();

        let mut withdrawal = Withdrawal::try_from_events(new_withdrawal.into_events()).unwrap();
        withdrawal
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();
        let _ = withdrawal.confirm(dummy_audit_info()).unwrap();
        withdrawal
    }

    #[test]
    fn can_revert_confirmed_withdrawal() {
        let mut withdrawal = create_confirmed_withdrawal();

        let result = withdrawal.revert(dummy_audit_info());

        assert!(result.is_ok());
        assert_eq!(withdrawal.status(), WithdrawalStatus::Reverted);
    }

    #[test]
    fn cancelled_withdrawal_is_ignored_on_revert() {
        let new_withdrawal = NewWithdrawal::builder()
            .id(WithdrawalId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ONE)
            .reference(None)
            .approval_process_id(ApprovalProcessId::new())
            .audit_info(dummy_audit_info())
            .build()
            .unwrap();

        let mut withdrawal = Withdrawal::try_from_events(new_withdrawal.into_events()).unwrap();
        withdrawal
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();
        let _ = withdrawal.cancel(dummy_audit_info()).unwrap();

        let result = withdrawal.revert(dummy_audit_info()).unwrap();
        assert!(result.was_ignored());
    }

    #[test]
    fn reverted_withdrawal_is_ignored_on_revert() {
        let mut withdrawal = create_confirmed_withdrawal();

        let _ = withdrawal.revert(dummy_audit_info()).unwrap();
        let result = withdrawal.revert(dummy_audit_info()).unwrap();
        assert!(result.was_ignored());
    }

    #[test]
    fn cannot_revert_unconfirmed_withdrawal() {
        let new_withdrawal = NewWithdrawal::builder()
            .id(WithdrawalId::new())
            .deposit_account_id(DepositAccountId::new())
            .amount(UsdCents::ONE)
            .reference(None)
            .approval_process_id(ApprovalProcessId::new())
            .audit_info(dummy_audit_info())
            .build()
            .unwrap();

        let mut withdrawal = Withdrawal::try_from_events(new_withdrawal.into_events()).unwrap();
        withdrawal
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();

        let result = withdrawal.revert(dummy_audit_info());

        assert!(matches!(result, Err(WithdrawalError::NotConfirmed(_))));
    }
}
