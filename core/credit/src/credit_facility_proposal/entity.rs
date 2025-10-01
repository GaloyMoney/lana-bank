use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use core_custody::CustodianId;

use crate::{primitives::*, terms::TermValues};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CreditFacilityProposalId")]
pub enum CreditFacilityProposalEvent {
    Initialized {
        id: CreditFacilityProposalId,
        customer_id: CustomerId,
        customer_type: CustomerType,
        custodian_id: Option<CustodianId>,
        approval_process_id: ApprovalProcessId,
        disbursal_credit_account_id: CalaAccountId,
        terms: TermValues,
        amount: UsdCents,
        status: CreditFacilityProposalStatus,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        status: CreditFacilityProposalStatus,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct CreditFacilityProposal {
    pub id: CreditFacilityProposalId,
    pub customer_id: CustomerId,
    pub customer_type: CustomerType,
    pub custodian_id: Option<CustodianId>,
    pub approval_process_id: ApprovalProcessId,
    pub disbursal_credit_account_id: CalaAccountId,
    pub amount: UsdCents,
    pub terms: TermValues,

    events: EntityEvents<CreditFacilityProposalEvent>,
}

impl CreditFacilityProposal {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn is_approval_process_concluded(&self) -> bool {
        self.events.iter_all().any(|e| {
            matches!(
                e,
                CreditFacilityProposalEvent::ApprovalProcessConcluded { .. }
            )
        })
    }

    pub(crate) fn approval_process_concluded(&mut self, approved: bool) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityProposalEvent::ApprovalProcessConcluded { .. }
        );

        let status = if approved {
            CreditFacilityProposalStatus::Approved
        } else {
            CreditFacilityProposalStatus::Denied
        };

        self.events
            .push(CreditFacilityProposalEvent::ApprovalProcessConcluded {
                approval_process_id: self.id.into(),
                status,
            });
        Idempotent::Executed(())
    }

    pub fn status(&self) -> CreditFacilityProposalStatus {
        self.events
            .iter_all()
            .rev()
            .map(|event| match event {
                CreditFacilityProposalEvent::ApprovalProcessConcluded { status, .. } => *status,
                CreditFacilityProposalEvent::Initialized { status, .. } => *status,
            })
            .next()
            .expect("status should always exist")
    }
}

impl TryFromEvents<CreditFacilityProposalEvent> for CreditFacilityProposal {
    fn try_from_events(
        events: EntityEvents<CreditFacilityProposalEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = CreditFacilityProposalBuilder::default();
        for event in events.iter_all() {
            match event {
                CreditFacilityProposalEvent::Initialized {
                    id,
                    customer_id,
                    customer_type,
                    custodian_id,
                    approval_process_id,
                    disbursal_credit_account_id,
                    terms,
                    amount,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .customer_id(*customer_id)
                        .customer_type(*customer_type)
                        .custodian_id(*custodian_id)
                        .approval_process_id(*approval_process_id)
                        .disbursal_credit_account_id(*disbursal_credit_account_id)
                        .terms(*terms)
                        .amount(*amount);
                }
                CreditFacilityProposalEvent::ApprovalProcessConcluded { .. } => {}
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewCreditFacilityProposal {
    #[builder(setter(into))]
    pub(super) id: CreditFacilityProposalId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    pub(super) customer_type: CustomerType,
    #[builder(setter(into, strip_option), default)]
    pub(super) custodian_id: Option<CustodianId>,
    #[builder(setter(into))]
    pub(super) approval_process_id: ApprovalProcessId,
    #[builder(setter(into))]
    pub(super) disbursal_credit_account_id: CalaAccountId,
    terms: TermValues,
    amount: UsdCents,
}

impl NewCreditFacilityProposal {
    pub fn builder() -> NewCreditFacilityProposalBuilder {
        NewCreditFacilityProposalBuilder::default()
    }
}

impl IntoEvents<CreditFacilityProposalEvent> for NewCreditFacilityProposal {
    fn into_events(self) -> EntityEvents<CreditFacilityProposalEvent> {
        EntityEvents::init(
            self.id,
            [CreditFacilityProposalEvent::Initialized {
                id: self.id,
                customer_id: self.customer_id,
                customer_type: self.customer_type,
                custodian_id: self.custodian_id,
                approval_process_id: self.approval_process_id,
                disbursal_credit_account_id: self.disbursal_credit_account_id,
                terms: self.terms,
                amount: self.amount,
                status: CreditFacilityProposalStatus::PendingApproval,
            }],
        )
    }
}
