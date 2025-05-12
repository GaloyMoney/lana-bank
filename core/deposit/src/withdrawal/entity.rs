use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::{
    ApprovalProcessId, CalaTransactionId, DepositAccountId, UsdCents, WithdrawalId,
};
use audit::AuditInfo;

use super::error::WithdrawalError;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
pub enum WithdrawalStatus {
    PendingApproval,
    PendingConfirmation,
    Confirmed,
    Denied,
    Cancelled,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "WithdrawalId")]
pub enum WithdrawalEvent {
    Initialized {
        id: WithdrawalId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: String,
        approval_process_id: ApprovalProcessId,
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

impl Withdrawal {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for deposit")
    }

    pub fn confirm(&mut self, audit_info: AuditInfo) -> Result<CalaTransactionId, WithdrawalError> {
        match self.is_approved_or_denied() {
            Some(false) => return Err(WithdrawalError::NotApproved(self.id)),
            None => return Err(WithdrawalError::NotApproved(self.id)),
            _ => (),
        }

        if self.is_confirmed() {
            return Err(WithdrawalError::AlreadyConfirmed(self.id));
        }

        if self.is_cancelled() {
            return Err(WithdrawalError::AlreadyCancelled(self.id));
        }

        let ledger_tx_id = CalaTransactionId::new();
        self.events.push(WithdrawalEvent::Confirmed {
            ledger_tx_id,
            audit_info,
        });

        Ok(ledger_tx_id)
    }

    pub fn cancel(&mut self, audit_info: AuditInfo) -> Result<CalaTransactionId, WithdrawalError> {
        if self.is_confirmed() {
            return Err(WithdrawalError::AlreadyConfirmed(self.id));
        }

        if self.is_cancelled() {
            return Err(WithdrawalError::AlreadyCancelled(self.id));
        }

        let ledger_tx_id = CalaTransactionId::new();
        self.events.push(WithdrawalEvent::Cancelled {
            ledger_tx_id,
            audit_info,
        });
        self.cancelled_tx_id = Some(ledger_tx_id);

        Ok(ledger_tx_id)
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
            .any(|e| matches!(e, WithdrawalEvent::Cancelled { .. }))
    }

    pub fn status(&self) -> WithdrawalStatus {
        if self.is_confirmed() {
            WithdrawalStatus::Confirmed
        } else if self.is_cancelled() {
            WithdrawalStatus::Cancelled
        } else {
            match self.is_approved_or_denied() {
                Some(true) => WithdrawalStatus::PendingConfirmation,
                Some(false) => WithdrawalStatus::Denied,
                None => WithdrawalStatus::PendingApproval,
            }
        }
    }

    pub fn approval_process_concluded(
        &mut self,
        approved: bool,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            WithdrawalEvent::ApprovalProcessConcluded { .. }
        );
        self.events.push(WithdrawalEvent::ApprovalProcessConcluded {
            approval_process_id: self.id.into(),
            approved,
            audit_info,
        });
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
                deposit_account_id: self.deposit_account_id,
                amount: self.amount,
                approval_process_id: self.approval_process_id,
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
}
