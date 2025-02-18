use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{
    audit::AuditInfo,
    credit_facility::{ledger::*, CreditFacilityAccountIds},
    primitives::*,
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DisbursalId")]
pub enum DisbursalEvent {
    Initialized {
        id: DisbursalId,
        approval_process_id: ApprovalProcessId,
        facility_id: CreditFacilityId,
        idx: DisbursalIdx,
        amount: UsdCents,
        account_ids: CreditFacilityAccountIds,
        deposit_account_id: DepositAccountId,
        audit_info: AuditInfo,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        approved: bool,
        audit_info: AuditInfo,
    },
    Settled {
        ledger_tx_id: LedgerTxId,
        audit_info: AuditInfo,
    },
    Cancelled {
        ledger_tx_id: LedgerTxId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Disbursal {
    pub id: DisbursalId,
    pub approval_process_id: ApprovalProcessId,
    pub facility_id: CreditFacilityId,
    pub idx: DisbursalIdx,
    pub amount: UsdCents,
    pub account_ids: CreditFacilityAccountIds,
    pub deposit_account_id: DepositAccountId,
    #[builder(setter(strip_option), default)]
    pub concluded_tx_id: Option<LedgerTxId>,
    pub(super) events: EntityEvents<DisbursalEvent>,
}

impl TryFromEvents<DisbursalEvent> for Disbursal {
    fn try_from_events(events: EntityEvents<DisbursalEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DisbursalBuilder::default();
        for event in events.iter_all() {
            match event {
                DisbursalEvent::Initialized {
                    id,
                    approval_process_id,
                    facility_id,
                    idx,
                    amount,
                    account_ids,
                    deposit_account_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .approval_process_id(*approval_process_id)
                        .facility_id(*facility_id)
                        .idx(*idx)
                        .amount(*amount)
                        .account_ids(*account_ids)
                        .deposit_account_id(*deposit_account_id)
                }
                DisbursalEvent::Settled { ledger_tx_id, .. } => {
                    builder = builder.concluded_tx_id(*ledger_tx_id)
                }
                DisbursalEvent::Cancelled { ledger_tx_id, .. } => {
                    builder = builder.concluded_tx_id(*ledger_tx_id)
                }
                DisbursalEvent::ApprovalProcessConcluded { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

impl Disbursal {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn is_approval_process_concluded(&self) -> bool {
        self.events
            .iter_all()
            .any(|e| matches!(e, DisbursalEvent::ApprovalProcessConcluded { .. }))
    }
    pub fn status(&self) -> DisbursalStatus {
        if self.is_confirmed() {
            DisbursalStatus::Confirmed
        } else {
            match self.is_approved() {
                Some(true) => DisbursalStatus::Approved,
                Some(false) => DisbursalStatus::Denied,
                None => DisbursalStatus::New,
            }
        }
    }

    pub(crate) fn approval_process_concluded(
        &mut self,
        approved: bool,
        audit_info: AuditInfo,
    ) -> Idempotent<DisbursalData> {
        idempotency_guard!(
            self.events.iter_all(),
            DisbursalEvent::ApprovalProcessConcluded { .. }
        );
        self.events.push(DisbursalEvent::ApprovalProcessConcluded {
            approval_process_id: self.approval_process_id,
            approved,
            audit_info: audit_info.clone(),
        });
        let tx_id = LedgerTxId::new();
        let data = DisbursalData {
            tx_ref: format!("disbursal-{}", self.id),
            tx_id,
            amount: self.amount,
            cancelled: !approved,
            credit_facility_account_ids: self.account_ids,
            debit_account_id: self.deposit_account_id.into(),
        };
        if approved {
            self.events.push(DisbursalEvent::Settled {
                ledger_tx_id: tx_id,
                audit_info,
            });
        } else {
            self.events.push(DisbursalEvent::Cancelled {
                ledger_tx_id: tx_id,
                audit_info,
            });
        }
        self.concluded_tx_id = Some(tx_id);

        Idempotent::Executed(data)
    }

    pub(super) fn is_approved(&self) -> Option<bool> {
        for event in self.events.iter_all() {
            if let DisbursalEvent::ApprovalProcessConcluded { approved, .. } = event {
                return Some(*approved);
            }
        }
        None
    }

    pub(super) fn is_confirmed(&self) -> bool {
        for event in self.events.iter_all() {
            match event {
                DisbursalEvent::Settled { .. } => return true,
                _ => continue,
            }
        }
        false
    }
}

#[derive(Debug, Builder)]
pub struct NewDisbursal {
    #[builder(setter(into))]
    pub(super) id: DisbursalId,
    #[builder(setter(into))]
    pub(crate) approval_process_id: ApprovalProcessId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    pub(super) idx: DisbursalIdx,
    pub(super) amount: UsdCents,
    pub(super) account_ids: CreditFacilityAccountIds,
    pub(super) deposit_account_id: DepositAccountId,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
}

impl NewDisbursal {
    pub fn builder() -> NewDisbursalBuilder {
        NewDisbursalBuilder::default()
    }
}

impl IntoEvents<DisbursalEvent> for NewDisbursal {
    fn into_events(self) -> EntityEvents<DisbursalEvent> {
        EntityEvents::init(
            self.id,
            [DisbursalEvent::Initialized {
                id: self.id,
                approval_process_id: self.approval_process_id,
                facility_id: self.credit_facility_id,
                idx: self.idx,
                amount: self.amount,
                account_ids: self.account_ids,
                deposit_account_id: self.deposit_account_id,
                audit_info: self.audit_info,
            }],
        )
    }
}
