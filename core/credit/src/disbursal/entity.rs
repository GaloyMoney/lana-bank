use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::{
    ledger::CreditFacilityAccountIds,
    obligation::{NewObligation, ObligationType},
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
        disbursal_credit_account_id: CalaAccountId,
        disbursal_due_date: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        approved: bool,
        audit_info: AuditInfo,
    },
    Settled {
        ledger_tx_id: LedgerTxId,
        obligation_id: ObligationId,
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
    pub disbursal_credit_account_id: CalaAccountId,
    pub disbursal_due_date: DateTime<Utc>,
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
                    disbursal_credit_account_id,
                    disbursal_due_date,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .approval_process_id(*approval_process_id)
                        .facility_id(*facility_id)
                        .idx(*idx)
                        .amount(*amount)
                        .account_ids(*account_ids)
                        .disbursal_credit_account_id(*disbursal_credit_account_id)
                        .disbursal_due_date(*disbursal_due_date)
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

    pub fn obligation_id(&self) -> Option<ObligationId> {
        self.events.iter_all().find_map(|event| match event {
            DisbursalEvent::Settled { obligation_id, .. } => Some(*obligation_id),
            _ => None,
        })
    }

    pub(crate) fn approval_process_concluded(
        &mut self,
        tx_id: LedgerTxId,
        approved: bool,
        audit_info: AuditInfo,
    ) -> Idempotent<Option<NewObligation>> {
        idempotency_guard!(
            self.events.iter_all(),
            DisbursalEvent::ApprovalProcessConcluded { .. }
        );
        self.events.push(DisbursalEvent::ApprovalProcessConcluded {
            approval_process_id: self.approval_process_id,
            approved,
            audit_info: audit_info.clone(),
        });
        let tx_ref: &str = &format!("disbursal-{}", self.id);
        let new_obligation = if approved {
            Some(self.settle_disbursal(tx_id, tx_ref, audit_info))
        } else {
            self.events.push(DisbursalEvent::Cancelled {
                ledger_tx_id: tx_id,
                audit_info,
            });
            None
        };
        self.concluded_tx_id = Some(tx_id);

        Idempotent::Executed(new_obligation)
    }

    pub(super) fn is_approved(&self) -> Option<bool> {
        for event in self.events.iter_all() {
            if let DisbursalEvent::ApprovalProcessConcluded { approved, .. } = event {
                return Some(*approved);
            }
        }
        None
    }

    fn settle_disbursal(
        &mut self,
        tx_id: LedgerTxId,
        tx_ref: &str,
        audit_info: AuditInfo,
    ) -> NewObligation {
        let obligation_id = ObligationId::new();
        self.events.push(DisbursalEvent::Settled {
            ledger_tx_id: tx_id,
            obligation_id,
            audit_info: audit_info.clone(),
        });

        NewObligation::builder()
            .id(obligation_id)
            .credit_facility_id(self.facility_id)
            .obligation_type(ObligationType::Disbursal)
            .reference(tx_ref.to_string())
            .amount(self.amount)
            .tx_id(tx_id)
            .account_to_be_debited_id(self.account_ids.disbursed_receivable_account_id)
            .account_to_be_credited_id(self.disbursal_credit_account_id)
            .due_date(self.disbursal_due_date)
            .overdue_date(self.disbursal_due_date)
            .recorded_at(crate::time::now())
            .audit_info(audit_info)
            .build()
            .expect("could not build new disbursal obligation")
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
    pub(super) disbursal_credit_account_id: CalaAccountId,
    pub(super) disbursal_due_date: DateTime<Utc>,
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
                disbursal_credit_account_id: self.disbursal_credit_account_id,
                disbursal_due_date: self.disbursal_due_date,
                audit_info: self.audit_info,
            }],
        )
    }
}
