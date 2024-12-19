use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::{
    ApprovalProcessId, DepositAccountId, LedgerTransactionId, UsdCents, WithdrawalId,
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
        audit_info: AuditInfo,
    },
    ApprovalProcessStarted {
        approval_process_id: ApprovalProcessId,
        audit_info: AuditInfo,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        approved: bool,
        audit_info: AuditInfo,
    },
    Confirmed {
        ledger_tx_id: LedgerTransactionId,
        audit_info: AuditInfo,
    },
    Cancelled {
        ledger_tx_id: LedgerTransactionId,
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

    pub(super) events: EntityEvents<WithdrawalEvent>,
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
    ) -> Result<LedgerTransactionId, WithdrawalError> {
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

        let ledger_tx_id = LedgerTransactionId::new();
        self.events.push(WithdrawalEvent::Confirmed {
            ledger_tx_id,
            audit_info,
        });

        Ok(ledger_tx_id)
    }

    pub fn cancel(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<LedgerTransactionId, WithdrawalError> {
        if self.is_confirmed() {
            return Err(WithdrawalError::AlreadyConfirmed(self.id));
        }

        if self.is_cancelled() {
            return Err(WithdrawalError::AlreadyCancelled(self.id));
        }

        let ledger_tx_id = LedgerTransactionId::new();
        self.events.push(WithdrawalEvent::Cancelled {
            ledger_tx_id,
            audit_info,
        });
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
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .deposit_account_id(*deposit_account_id)
                        .amount(*amount)
                        .reference(reference.clone());
                }
                WithdrawalEvent::ApprovalProcessStarted {
                    approval_process_id,
                    ..
                } => builder = builder.approval_process_id(*approval_process_id),
                _ => {}
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
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

impl IntoEvents<WithdrawalEvent> for NewWithdrawal {
    fn into_events(self) -> EntityEvents<WithdrawalEvent> {
        EntityEvents::init(
            self.id,
            [
                WithdrawalEvent::Initialized {
                    reference: self.reference(),
                    id: self.id,
                    deposit_account_id: self.deposit_account_id,
                    amount: self.amount,
                    audit_info: self.audit_info.clone(),
                },
                WithdrawalEvent::ApprovalProcessStarted {
                    approval_process_id: self.approval_process_id,
                    audit_info: self.audit_info,
                },
            ],
        )
    }
}
