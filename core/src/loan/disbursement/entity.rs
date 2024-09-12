use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use crate::{
    entity::*,
    ledger::{
        customer::CustomerLedgerAccountIds, disbursement::DisbursementData, loan::LoanAccountIds,
    },
    primitives::*,
};

use super::DisbursementError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DisbursementEvent {
    Initialized {
        id: DisbursementId,
        loan_id: LoanId,
        idx: DisbursementIdx,
        amount: UsdCents,
        account_ids: LoanAccountIds,
        customer_account_ids: CustomerLedgerAccountIds,
        audit_info: AuditInfo,
    },
    ApprovalAdded {
        id: DisbursementId,
        approving_user_id: UserId,
        approving_user_roles: HashSet<Role>,
        audit_info: AuditInfo,
    },
    Approved {
        tx_id: LedgerTxId,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    Concluded {
        id: DisbursementId,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
}

impl EntityEvent for DisbursementEvent {
    type EntityId = DisbursementId;
    fn event_table_name() -> &'static str {
        "disbursement_events"
    }
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Disbursement {
    pub id: DisbursementId,
    pub loan_id: LoanId,
    pub idx: DisbursementIdx,
    pub amount: UsdCents,
    pub account_ids: LoanAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
    pub(super) events: EntityEvents<DisbursementEvent>,
}

impl Entity for Disbursement {
    type Event = DisbursementEvent;
}

impl TryFrom<EntityEvents<DisbursementEvent>> for Disbursement {
    type Error = EntityError;

    fn try_from(events: EntityEvents<DisbursementEvent>) -> Result<Self, Self::Error> {
        let mut builder = DisbursementBuilder::default();
        for event in events.iter() {
            match event {
                DisbursementEvent::Initialized {
                    id,
                    loan_id,
                    idx,
                    amount,
                    account_ids,
                    customer_account_ids,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .loan_id(*loan_id)
                        .idx(*idx)
                        .amount(*amount)
                        .account_ids(*account_ids)
                        .customer_account_ids(*customer_account_ids)
                }
                DisbursementEvent::Concluded { .. } => (),
                DisbursementEvent::ApprovalAdded { .. } => (),
                DisbursementEvent::Approved { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

impl Disbursement {
    pub fn conclude(&mut self, recorded_at: DateTime<Utc>, audit_info: AuditInfo) {
        let is_concluded = self
            .events
            .iter()
            .any(|event| matches!(event, DisbursementEvent::Concluded { .. }));

        if !is_concluded {
            self.events.push(DisbursementEvent::Concluded {
                id: self.id,
                recorded_at,
                audit_info,
            })
        }
    }

    pub fn is_concluded(&self) -> bool {
        self.events
            .iter()
            .any(|event| matches!(event, DisbursementEvent::Concluded { .. }))
    }

    fn has_user_previously_approved(&self, user_id: UserId) -> bool {
        for event in self.events.iter() {
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

    fn approval_threshold_met(&self) -> bool {
        let mut n_admin = 0;
        let mut n_bank_manager = 0;

        for event in self.events.iter() {
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
        for event in self.events.iter() {
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
            id: self.id,
            approving_user_id,
            approving_user_roles,
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
        DisbursementData { tx_id, .. }: DisbursementData,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) {
        self.events.push(DisbursementEvent::Approved {
            tx_id,
            audit_info,
            recorded_at: executed_at,
        });
    }
}

#[derive(Debug)]
pub struct NewDisbursement {
    pub(super) id: DisbursementId,
    pub(super) loan_id: LoanId,
    pub(super) idx: DisbursementIdx,
    pub(super) amount: UsdCents,
    pub(super) account_ids: LoanAccountIds,
    pub(super) customer_account_ids: CustomerLedgerAccountIds,
    pub(super) audit_info: AuditInfo,
}

impl NewDisbursement {
    pub fn new(
        loan_id: LoanId,
        idx: DisbursementIdx,
        amount: UsdCents,
        account_ids: LoanAccountIds,
        customer_account_ids: CustomerLedgerAccountIds,
        audit_info: AuditInfo,
    ) -> Self {
        Self {
            id: DisbursementId::new(),
            loan_id,
            idx,
            amount,
            account_ids,
            customer_account_ids,
            audit_info,
        }
    }

    pub(super) fn initial_events(self) -> EntityEvents<DisbursementEvent> {
        EntityEvents::init(
            self.id,
            [DisbursementEvent::Initialized {
                id: self.id,
                loan_id: self.loan_id,
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

    use super::*;

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: Subject::from(UserId::new()),
        }
    }

    fn init_events() -> EntityEvents<DisbursementEvent> {
        EntityEvents::init(
            DisbursementId::new(),
            [DisbursementEvent::Initialized {
                id: DisbursementId::new(),
                loan_id: LoanId::new(),
                idx: DisbursementIdx::FIRST,
                amount: UsdCents::from(1000_00),
                account_ids: LoanAccountIds::new(),
                customer_account_ids: CustomerLedgerAccountIds::new(),
                audit_info: dummy_audit_info(),
            }],
        )
    }

    #[test]
    fn admin_and_bank_manager_can_approve() {
        let mut disbursement = Disbursement::try_from(init_events()).unwrap();
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
        let mut disbursement = Disbursement::try_from(init_events()).unwrap();
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
        let mut disbursement = Disbursement::try_from(init_events()).unwrap();
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
        let mut disbursement = Disbursement::try_from(init_events()).unwrap();
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
