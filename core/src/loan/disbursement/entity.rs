use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::*,
    primitives::{AuditInfo, DisbursementId, DisbursementIdx, LoanId, UsdCents},
};

crate::entity_id! { DisbursementDbRef }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DisbursementEvent {
    Initialized {
        id: DisbursementDbRef,
        loan_id: LoanId,
        idx: DisbursementIdx,
        amount: UsdCents,
        audit_info: AuditInfo,
    },
}

impl EntityEvent for DisbursementEvent {
    type EntityId = DisbursementDbRef;
    fn event_table_name() -> &'static str {
        "disbursement_events"
    }
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Disbursement {
    pub(super) _id: DisbursementDbRef,
    pub id: DisbursementId,
    pub(super) _events: EntityEvents<DisbursementEvent>,
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
                    id, loan_id, idx, ..
                } => {
                    builder = builder._id(*id).id(DisbursementId {
                        loan_id: *loan_id,
                        idx: *idx,
                    })
                }
            }
        }
        builder._events(events).build()
    }
}

#[derive(Debug)]
pub struct NewDisbursement {
    pub(super) id: DisbursementDbRef,
    pub(super) loan_id: LoanId,
    pub(super) idx: DisbursementIdx,
    pub(super) amount: UsdCents,
    pub(super) audit_info: AuditInfo,
}

impl NewDisbursement {
    pub fn new(
        audit_info: AuditInfo,
        loan_id: LoanId,
        idx: DisbursementIdx,
        amount: UsdCents,
    ) -> Self {
        Self {
            id: DisbursementDbRef::new(),
            loan_id,
            idx,
            amount,
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
                audit_info: self.audit_info,
            }],
        )
    }
}
