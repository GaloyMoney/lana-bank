use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use audit::AuditInfo;

use crate::primitives::{ApprovalProcessId, DepositAccountId, LedgerTransactionId, WithdrawalId};

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
    pub approval_process_id: ApprovalProcessId,
    pub(super) events: EntityEvents<WithdrawalEvent>,
}

impl Withdrawal {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for deposit")
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
                WithdrawalEvent::ApprovalProcessStarted {
                    approval_process_id: self.approval_process_id,
                    audit_info: self.audit_info,
                },
            ],
        )
    }
}
