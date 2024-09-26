use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use crate::{
    credit_facility::CreditFacilityAccountIds,
    entity::*,
    ledger::{customer::CustomerLedgerAccountIds, disbursement::DisbursementData},
    primitives::*,
};

use super::DisbursementError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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
        audit_info: AuditInfo,
    },
    Approved {
        tx_id: LedgerTxId,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    Concluded {
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
    pub facility_id: CreditFacilityId,
    pub idx: DisbursementIdx,
    pub amount: UsdCents,
    pub account_ids: CreditFacilityAccountIds,
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
                DisbursementEvent::Concluded { .. } => (),
                DisbursementEvent::ApprovalAdded { .. } => (),
                DisbursementEvent::Approved { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

impl Disbursement {
    fn is_concluded(&self) -> bool {
        self.events
            .iter()
            .any(|event| matches!(event, DisbursementEvent::Concluded { .. }))
    }

    pub fn conclude(&mut self, recorded_at: DateTime<Utc>, audit_info: AuditInfo) {
        if !self.is_concluded() {
            self.events.push(DisbursementEvent::Concluded {
                recorded_at,
                audit_info,
            })
        }
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
}

#[derive(Debug)]
pub struct NewDisbursement {
    pub(super) id: DisbursementId,
    pub(super) facility_id: CreditFacilityId,
    pub(super) idx: DisbursementIdx,
    pub(super) amount: UsdCents,
    pub(super) account_ids: CreditFacilityAccountIds,
    pub(super) customer_account_ids: CustomerLedgerAccountIds,
    pub(super) audit_info: AuditInfo,
}

impl NewDisbursement {
    pub fn new(
        facility_id: CreditFacilityId,
        idx: DisbursementIdx,
        amount: UsdCents,
        account_ids: CreditFacilityAccountIds,
        customer_account_ids: CustomerLedgerAccountIds,
        audit_info: AuditInfo,
    ) -> Self {
        Self {
            id: DisbursementId::new(),
            facility_id,
            idx,
            amount,
            account_ids,
            customer_account_ids,
            audit_info,
        }
    }

    pub fn initial_events(self) -> EntityEvents<DisbursementEvent> {
        EntityEvents::init(
            self.id,
            [DisbursementEvent::Initialized {
                id: self.id,
                facility_id: self.facility_id,
                idx: self.idx,
                amount: self.amount,
                account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
                audit_info: self.audit_info,
            }],
        )
    }
}
