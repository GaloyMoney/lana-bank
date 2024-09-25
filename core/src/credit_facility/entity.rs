use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use crate::{
    entity::*,
    ledger::{
        credit_facility::{CreditFacilityAccountIds, CreditFacilityApprovalData},
        customer::CustomerLedgerAccountIds,
    },
    primitives::*,
};

use super::CreditFacilityError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreditFacilityEvent {
    Initialized {
        id: CreditFacilityId,
        customer_id: CustomerId,
        facility: UsdCents,
        account_ids: CreditFacilityAccountIds,
        customer_account_ids: CustomerLedgerAccountIds,
        audit_info: AuditInfo,
    },
    ApprovalAdded {
        approving_user_id: UserId,
        approving_user_roles: HashSet<Role>,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    Approved {
        tx_id: LedgerTxId,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
}

impl EntityEvent for CreditFacilityEvent {
    type EntityId = CreditFacilityId;
    fn event_table_name() -> &'static str {
        "credit_facility_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct CreditFacility {
    pub id: CreditFacilityId,
    pub customer_id: CustomerId,
    pub account_ids: CreditFacilityAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
    pub(super) events: EntityEvents<CreditFacilityEvent>,
}

impl Entity for CreditFacility {
    type Event = CreditFacilityEvent;
}

impl CreditFacility {
    fn facility(&self) -> UsdCents {
        for event in self.events.iter() {
            match event {
                CreditFacilityEvent::Initialized { facility, .. } => return *facility,
                _ => continue,
            }
        }
        UsdCents::ZERO
    }

    pub(super) fn is_approved(&self) -> bool {
        for event in self.events.iter() {
            match event {
                CreditFacilityEvent::Approved { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    fn approval_threshold_met(&self) -> bool {
        let mut n_admin = 0;
        let mut n_bank_manager = 0;

        for event in self.events.iter() {
            if let CreditFacilityEvent::ApprovalAdded {
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

    fn has_user_previously_approved(&self, user_id: UserId) -> bool {
        for event in self.events.iter() {
            match event {
                CreditFacilityEvent::ApprovalAdded {
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

    pub(super) fn add_approval(
        &mut self,
        approving_user_id: UserId,
        approving_user_roles: HashSet<Role>,
        audit_info: AuditInfo,
    ) -> Result<Option<CreditFacilityApprovalData>, CreditFacilityError> {
        if self.has_user_previously_approved(approving_user_id) {
            return Err(CreditFacilityError::UserCannotApproveTwice);
        }

        if self.is_approved() {
            return Err(CreditFacilityError::AlreadyApproved);
        }

        self.events.push(CreditFacilityEvent::ApprovalAdded {
            approving_user_id,
            approving_user_roles,
            audit_info,
            recorded_at: Utc::now(),
        });

        if self.approval_threshold_met() {
            let tx_ref = format!("{}-approval", self.id);
            Ok(Some(CreditFacilityApprovalData {
                facility: self.facility(),
                tx_ref,
                tx_id: LedgerTxId::new(),
                credit_facility_account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
            }))
        } else {
            Ok(None)
        }
    }

    pub(super) fn confirm_approval(
        &mut self,
        CreditFacilityApprovalData { tx_id, .. }: CreditFacilityApprovalData,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) {
        self.events.push(CreditFacilityEvent::Approved {
            tx_id,
            audit_info,
            recorded_at: executed_at,
        });
    }
}

impl TryFrom<EntityEvents<CreditFacilityEvent>> for CreditFacility {
    type Error = EntityError;

    fn try_from(events: EntityEvents<CreditFacilityEvent>) -> Result<Self, Self::Error> {
        let mut builder = CreditFacilityBuilder::default();
        for event in events.iter() {
            match event {
                CreditFacilityEvent::Initialized {
                    id, customer_id, ..
                } => builder = builder.id(*id).customer_id(*customer_id),
                CreditFacilityEvent::Approved { .. } => (),
                CreditFacilityEvent::ApprovalAdded { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewCreditFacility {
    #[builder(setter(into))]
    pub(super) id: CreditFacilityId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    facility: UsdCents,
    account_ids: CreditFacilityAccountIds,
    customer_account_ids: CustomerLedgerAccountIds,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
}

impl NewCreditFacility {
    pub fn builder() -> NewCreditFacilityBuilder {
        NewCreditFacilityBuilder::default()
    }

    pub(super) fn initial_events(self) -> EntityEvents<CreditFacilityEvent> {
        EntityEvents::init(
            self.id,
            [CreditFacilityEvent::Initialized {
                id: self.id,
                audit_info: self.audit_info,
                customer_id: self.customer_id,
                facility: self.facility,
                account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
            }],
        )
    }
}
