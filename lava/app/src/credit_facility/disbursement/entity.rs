use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

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
        facility_id: CreditFacilityId,
        idx: DisbursementIdx,
        amount: UsdCents,
        account_ids: CreditFacilityAccountIds,
        customer_account_ids: CustomerLedgerAccountIds,
        audit_info: AuditInfo,
    },
    ApprovalAdded {
        approving_user_id: UserId,
        approving_user_roles: HashSet<Role>,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    Approved {
        tx_id: LedgerTxId,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Disbursement {
    pub id: DisbursementId,
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
                    facility_id,
                    idx,
                    amount,
                    account_ids,
                    customer_account_ids,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .facility_id(*facility_id)
                        .idx(*idx)
                        .amount(*amount)
                        .account_ids(*account_ids)
                        .customer_account_ids(*customer_account_ids)
                }
                DisbursementEvent::ApprovalAdded { .. } => (),
                DisbursementEvent::Approved { .. } => (),
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

    fn has_user_previously_approved(&self, user_id: UserId) -> bool {
        for event in self.events.iter_all() {
            match event {
                DisbursementEvent::ApprovalAdded {
                    approving_user_id, ..
                } => {
                    if user_id == *approving_user_id {
                        return true;
                    }
                }
                _ => continue,
            }
        }
        false
    }

    pub fn status(&self) -> DisbursementStatus {
        if self.is_approved() {
            DisbursementStatus::Approved
        } else {
            DisbursementStatus::New
        }
    }

    fn approval_threshold_met(&self) -> bool {
        let mut n_admin = 0;
        let mut n_bank_manager = 0;

        for event in self.events.iter_all() {
            if let DisbursementEvent::ApprovalAdded {
                approving_user_roles,
                ..
            } = event
            {
                if approving_user_roles.contains(&Role::Superuser) {
                    return true;
                } else if approving_user_roles.contains(&Role::Admin) {
                    n_admin += 1;
                } else {
                    n_bank_manager += 1;
                }
            }
        }

        n_admin >= 1 && n_admin + n_bank_manager >= 2
    }

    pub fn is_approved(&self) -> bool {
        for event in self.events.iter_all() {
            match event {
                DisbursementEvent::Approved { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    pub fn add_approval(
        &mut self,
        approving_user_id: UserId,
        approving_user_roles: HashSet<Role>,
        audit_info: AuditInfo,
    ) -> Result<Option<DisbursementData>, DisbursementError> {
        if self.has_user_previously_approved(approving_user_id) {
            return Err(DisbursementError::UserCannotApproveTwice);
        }

        if self.is_approved() {
            return Err(DisbursementError::AlreadyApproved);
        }

        self.events.push(DisbursementEvent::ApprovalAdded {
            approving_user_id,
            approving_user_roles,
            recorded_at: Utc::now(),
            audit_info,
        });

        if self.approval_threshold_met() {
            return Ok(Some(DisbursementData {
                tx_ref: format!("disbursement-{}", self.id),
                tx_id: LedgerTxId::new(),
                amount: self.amount,
                account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
            }));
        }
        Ok(None)
    }

    pub fn confirm_approval(
        &mut self,
        &DisbursementData { tx_id, .. }: &DisbursementData,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) {
        self.events.push(DisbursementEvent::Approved {
            tx_id,
            audit_info,
            recorded_at: executed_at,
        });
    }

    pub fn approvals(&self) -> Vec<DisbursementApproval> {
        let mut approvals = Vec::new();
        for event in self.events.iter_all() {
            if let DisbursementEvent::ApprovalAdded {
                approving_user_id,
                recorded_at,
                ..
            } = event
            {
                approvals.push(DisbursementApproval {
                    user_id: *approving_user_id,
                    approved_at: *recorded_at,
                });
            }
        }
        approvals
    }
}

#[derive(Debug, Builder)]
pub struct NewDisbursement {
    #[builder(setter(into))]
    pub(super) id: DisbursementId,
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

    fn init_events() -> EntityEvents<DisbursementEvent> {
        EntityEvents::init(
            DisbursementId::new(),
            [DisbursementEvent::Initialized {
                id: DisbursementId::new(),
                facility_id: CreditFacilityId::new(),
                idx: DisbursementIdx::FIRST,
                amount: UsdCents::from(100_000),
                account_ids: CreditFacilityAccountIds::new(),
                customer_account_ids: CustomerLedgerAccountIds::new(),
                audit_info: dummy_audit_info(),
            }],
        )
    }

    #[test]
    fn admin_and_bank_manager_can_approve() {
        let mut disbursement = Disbursement::try_from_events(init_events()).unwrap();
        let _admin_approval = disbursement.add_approval(
            UserId::new(),
            [Role::Admin].into_iter().collect(),
            dummy_audit_info(),
        );
        let _bank_manager_approval = disbursement.add_approval(
            UserId::new(),
            [Role::BankManager].into_iter().collect(),
            dummy_audit_info(),
        );

        assert!(disbursement.approval_threshold_met());
    }

    #[test]
    fn two_admin_can_approve() {
        let mut disbursement = Disbursement::try_from_events(init_events()).unwrap();
        let _first_admin_approval = disbursement.add_approval(
            UserId::new(),
            [Role::Admin].into_iter().collect(),
            dummy_audit_info(),
        );
        let _second_admin_approval = disbursement.add_approval(
            UserId::new(),
            [Role::Admin].into_iter().collect(),
            dummy_audit_info(),
        );

        assert!(disbursement.approval_threshold_met());
    }

    #[test]
    fn user_cannot_approve_twice() {
        let mut disbursement = Disbursement::try_from_events(init_events()).unwrap();
        let user_id = UserId::new();
        let first_approval = disbursement.add_approval(
            user_id,
            [Role::Admin].into_iter().collect(),
            dummy_audit_info(),
        );
        assert!(first_approval.is_ok());

        let result = disbursement.add_approval(
            user_id,
            [Role::Admin].into_iter().collect(),
            dummy_audit_info(),
        );

        assert!(matches!(
            result,
            Err(DisbursementError::UserCannotApproveTwice)
        ));
    }

    #[test]
    fn two_bank_managers_cannot_approve() {
        let mut disbursement = Disbursement::try_from_events(init_events()).unwrap();
        let first_bank_manager_approval = disbursement.add_approval(
            UserId::new(),
            [Role::BankManager].into_iter().collect(),
            dummy_audit_info(),
        );
        assert!(first_bank_manager_approval.is_ok());

        let second_bank_manager_approval = disbursement.add_approval(
            UserId::new(),
            [Role::BankManager].into_iter().collect(),
            dummy_audit_info(),
        );
        assert!(second_bank_manager_approval.is_ok());

        assert!(!disbursement.approval_threshold_met());
    }
}
