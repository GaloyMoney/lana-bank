use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{
    audit::AuditInfo, credit_facility::CreditFacilityAccountIds, customer::CustomerAccountIds,
    ledger::credit_facility::*, primitives::*,
};

use super::DisbursalError;

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
        customer_account_ids: CustomerAccountIds,
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
pub struct Disbursal {
    pub id: DisbursalId,
    pub approval_process_id: ApprovalProcessId,
    pub facility_id: CreditFacilityId,
    pub idx: DisbursalIdx,
    pub amount: UsdCents,
    pub account_ids: CreditFacilityAccountIds,
    pub customer_account_ids: CustomerAccountIds,
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
                DisbursalEvent::ApprovalProcessConcluded { .. } => (),
                DisbursalEvent::Confirmed { .. } => (),
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
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            DisbursalEvent::ApprovalProcessConcluded { .. }
        );
        self.events.push(DisbursalEvent::ApprovalProcessConcluded {
            approval_process_id: self.approval_process_id,
            approved,
            audit_info,
        });
        Idempotent::Executed(())
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
                DisbursalEvent::Confirmed { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    pub fn record(
        &mut self,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) -> Result<DisbursalData, DisbursalError> {
        if self.is_confirmed() {
            return Err(DisbursalError::AlreadyConfirmed);
        }

        match self.is_approved() {
            None => return Err(DisbursalError::ApprovalInProgress),
            Some(false) => return Err(DisbursalError::Denied),
            _ => (),
        }

        let tx_id = LedgerTxId::new();

        self.events.push(DisbursalEvent::Confirmed {
            ledger_tx_id: tx_id,
            confirmed_at: executed_at,
            audit_info,
        });

        Ok(DisbursalData {
            tx_ref: format!("disbursal-{}", self.id),
            tx_id,
            amount: self.amount,
            credit_facility_account_ids: self.account_ids,
            debit_account_id: self.customer_account_ids.deposit_account_id,
        })
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
    pub(super) customer_account_ids: CustomerAccountIds,
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

    fn disbursal_from(events: Vec<DisbursalEvent>) -> Disbursal {
        Disbursal::try_from_events(EntityEvents::init(DisbursalId::new(), events)).unwrap()
    }

    fn initial_events() -> Vec<DisbursalEvent> {
        let id = DisbursalId::new();
        vec![DisbursalEvent::Initialized {
            id,
            approval_process_id: id.into(),
            facility_id: CreditFacilityId::new(),
            idx: DisbursalIdx::FIRST,
            amount: UsdCents::from(100_000),
            account_ids: CreditFacilityAccountIds::new(),
            customer_account_ids: CustomerAccountIds::new(),
            audit_info: dummy_audit_info(),
        }]
    }

    #[test]
    fn errors_when_not_confirmed_yet() {
        let mut disbursal = disbursal_from(initial_events());
        assert!(matches!(
            disbursal.record(Utc::now(), dummy_audit_info()),
            Err(DisbursalError::ApprovalInProgress)
        ));
    }

    #[test]
    fn errors_if_denied() {
        let mut events = initial_events();
        events.push(DisbursalEvent::ApprovalProcessConcluded {
            approval_process_id: ApprovalProcessId::new(),
            approved: false,
            audit_info: dummy_audit_info(),
        });
        let mut disbursal = disbursal_from(events);

        assert!(matches!(
            disbursal.record(Utc::now(), dummy_audit_info()),
            Err(DisbursalError::Denied)
        ));
    }

    #[test]
    fn errors_if_already_confirmed() {
        let mut events = initial_events();
        events.extend([
            DisbursalEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: true,
                audit_info: dummy_audit_info(),
            },
            DisbursalEvent::Confirmed {
                ledger_tx_id: LedgerTxId::new(),
                confirmed_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let mut disbursal = disbursal_from(events);

        assert!(matches!(
            disbursal.record(Utc::now(), dummy_audit_info()),
            Err(DisbursalError::AlreadyConfirmed)
        ));
    }

    #[test]
    fn can_confirm() {
        let mut events = initial_events();
        events.extend([DisbursalEvent::ApprovalProcessConcluded {
            approval_process_id: ApprovalProcessId::new(),
            approved: true,
            audit_info: dummy_audit_info(),
        }]);
        let mut disbursal = disbursal_from(events);

        assert!(disbursal.record(Utc::now(), dummy_audit_info()).is_ok(),);
    }
}
