use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{
    audit::AuditInfo,
    credit_facility::CreditFacilityAccountIds,
    ledger::{customer::CustomerLedgerAccountIds, disbursement::DisbursementData},
    primitives::*,
};

use super::DisbursementError;

pub struct DisbursementApproval {
    pub user_id: UserId,
    pub approved_at: DateTime<Utc>,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DisbursementId")]
pub enum DisbursementEvent {
    Initialized {
        id: DisbursementId,
        approval_process_id: ApprovalProcessId,
        facility_id: CreditFacilityId,
        idx: DisbursementIdx,
        amount: UsdCents,
        account_ids: CreditFacilityAccountIds,
        customer_account_ids: CustomerLedgerAccountIds,
        audit_info: AuditInfo,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        approved: bool,
        audit_info: AuditInfo,
    },
    Confirmed {
        ledger_tx_id: LedgerTxId,
        audit_info: AuditInfo,
        confirmed_at: DateTime<Utc>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Disbursement {
    pub id: DisbursementId,
    pub approval_process_id: ApprovalProcessId,
    pub facility_id: CreditFacilityId,
    pub idx: DisbursementIdx,
    pub amount: UsdCents,
    pub account_ids: CreditFacilityAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
    pub(super) events: EntityEvents<DisbursementEvent>,
}

impl TryFromEvents<DisbursementEvent> for Disbursement {
    fn try_from_events(events: EntityEvents<DisbursementEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DisbursementBuilder::default();
        for event in events.iter_all() {
            match event {
                DisbursementEvent::Initialized {
                    id,
                    approval_process_id,
                    facility_id,
                    idx,
                    amount,
                    account_ids,
                    customer_account_ids,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .approval_process_id(*approval_process_id)
                        .facility_id(*facility_id)
                        .idx(*idx)
                        .amount(*amount)
                        .account_ids(*account_ids)
                        .customer_account_ids(*customer_account_ids)
                }
                DisbursementEvent::ApprovalProcessConcluded { .. } => (),
                DisbursementEvent::Confirmed { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

impl Disbursement {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn status(&self) -> DisbursementStatus {
        if self.is_confirmed() {
            DisbursementStatus::Confirmed
        } else {
            match self.is_approved() {
                Some(true) => DisbursementStatus::Approved,
                Some(false) => DisbursementStatus::Denied,
                None => DisbursementStatus::New,
            }
        }
    }

    pub(super) fn is_approved(&self) -> Option<bool> {
        for event in self.events.iter_all() {
            if let DisbursementEvent::ApprovalProcessConcluded { approved, .. } = event {
                return Some(*approved);
            }
        }
        None
    }
    pub(super) fn is_confirmed(&self) -> bool {
        for event in self.events.iter_all() {
            match event {
                DisbursementEvent::Confirmed { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    pub fn disbursement_data(&self) -> Result<DisbursementData, DisbursementError> {
        if self.is_confirmed() {
            return Err(DisbursementError::AlreadyConfirmed);
        }

        match self.is_approved() {
            None => return Err(DisbursementError::ApprovalInProgress),
            Some(false) => return Err(DisbursementError::Denied),
            _ => (),
        }

        Ok(DisbursementData {
            tx_ref: format!("disbursement-{}", self.id),
            tx_id: LedgerTxId::new(),
            amount: self.amount,
            account_ids: self.account_ids,
            customer_account_ids: self.customer_account_ids,
        })
    }

    pub fn confirm(
        &mut self,
        &DisbursementData { tx_id, .. }: &DisbursementData,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) {
        self.events.push(DisbursementEvent::Confirmed {
            ledger_tx_id: tx_id,
            confirmed_at: executed_at,
            audit_info,
        });
    }
}

#[derive(Debug, Builder)]
pub struct NewDisbursement {
    #[builder(setter(into))]
    pub(super) id: DisbursementId,
    #[builder(setter(into))]
    pub(crate) approval_process_id: ApprovalProcessId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    pub(super) idx: DisbursementIdx,
    pub(super) amount: UsdCents,
    pub(super) account_ids: CreditFacilityAccountIds,
    pub(super) customer_account_ids: CustomerLedgerAccountIds,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
}

impl NewDisbursement {
    pub fn builder() -> NewDisbursementBuilder {
        NewDisbursementBuilder::default()
    }
}

impl IntoEvents<DisbursementEvent> for NewDisbursement {
    fn into_events(self) -> EntityEvents<DisbursementEvent> {
        EntityEvents::init(
            self.id,
            [DisbursementEvent::Initialized {
                id: self.id,
                approval_process_id: self.approval_process_id,
                facility_id: self.credit_facility_id,
                idx: self.idx,
                amount: self.amount,
                account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use crate::audit::AuditEntryId;

    use super::*;

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn disbursement_from(events: &Vec<DisbursementEvent>) -> Disbursement {
        Disbursement::try_from_events(EntityEvents::init(DisbursementId::new(), events.clone()))
            .unwrap()
    }

    fn initial_events() -> Vec<DisbursementEvent> {
        let id = DisbursementId::new();
        vec![DisbursementEvent::Initialized {
            id,
            approval_process_id: id.into(),
            facility_id: CreditFacilityId::new(),
            idx: DisbursementIdx::FIRST,
            amount: UsdCents::from(100_000),
            account_ids: CreditFacilityAccountIds::new(),
            customer_account_ids: CustomerLedgerAccountIds::new(),
            audit_info: dummy_audit_info(),
        }]
    }

    #[test]
    fn errors_when_not_confirmed_yet() {
        let disbursement = disbursement_from(&initial_events());
        assert!(matches!(
            disbursement.disbursement_data(),
            Err(DisbursementError::ApprovalInProgress)
        ));
    }

    #[test]
    fn errors_if_denied() {
        let mut events = initial_events();
        events.push(DisbursementEvent::ApprovalProcessConcluded {
            approval_process_id: ApprovalProcessId::new(),
            approved: false,
            audit_info: dummy_audit_info(),
        });
        let disbursement = disbursement_from(&events);

        assert!(matches!(
            disbursement.disbursement_data(),
            Err(DisbursementError::Denied)
        ));
    }

    #[test]
    fn errors_if_already_activated() {
        let mut events = initial_events();
        events.extend([
            DisbursementEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: true,
                audit_info: dummy_audit_info(),
            },
            DisbursementEvent::Confirmed {
                ledger_tx_id: LedgerTxId::new(),
                confirmed_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let disbursement = disbursement_from(&events);

        assert!(matches!(
            disbursement.disbursement_data(),
            Err(DisbursementError::AlreadyActivated)
        ));
    }

    #[test]
    fn can_confirm() {
        let mut events = initial_events();
        events.extend([DisbursementEvent::ApprovalProcessConcluded {
            approval_process_id: ApprovalProcessId::new(),
            approved: true,
            audit_info: dummy_audit_info(),
        }]);
        let disbursement = disbursement_from(&events);

        assert!(disbursement.disbursement_data().is_ok(),);
    }
}
