use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

#[cfg(feature = "governance")]
use crate::primitives::ApprovalProcessId;
use crate::primitives::{DepositAccountId, LedgerTransactionId, WithdrawalId};
use audit::AuditInfo;

use super::error::WithdrawalError;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "WithdrawalId")]
pub enum WithdrawalEvent {
    Initialized {
        id: WithdrawalId,
        deposit_account_id: DepositAccountId,
        reference: String,
        audit_info: AuditInfo,
    },
    #[cfg(feature = "governance")]
    ApprovalProcessStarted {
        approval_process_id: ApprovalProcessId,
        audit_info: AuditInfo,
    },
    #[cfg(feature = "governance")]
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
    #[cfg(feature = "governance")]
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

    fn is_confirmed(&self) -> bool {
        self.events
            .iter_all()
            .any(|e| matches!(e, WithdrawalEvent::Confirmed { .. }))
    }

    pub(super) fn is_approved_or_denied(&self) -> Option<bool> {
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
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .deposit_account_id(*deposit_account_id)
                        .reference(reference.clone());
                }
                #[cfg(feature = "governance")]
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
    #[cfg(feature = "governance")]
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
                    audit_info: self.audit_info.clone(),
                },
                #[cfg(feature = "governance")]
                WithdrawalEvent::ApprovalProcessStarted {
                    approval_process_id: self.approval_process_id,
                    audit_info: self.audit_info,
                },
            ],
        )
    }
}
