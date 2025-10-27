use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use core_custody::CustodianId;
use es_entity::*;

use crate::{
    pending_credit_facility::{NewPendingCreditFacility, NewPendingCreditFacilityBuilder},
    primitives::*,
    terms::TermValues,
};

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
        disbursal_credit_account_id: CalaAccountId,
        terms: TermValues,
        amount: UsdCents,
        status: CreditFacilityProposalStatus,
    },
    CustomerApprovalConcluded {
        status: CreditFacilityProposalStatus,
        approval_process_id: Option<ApprovalProcessId>,
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
    #[builder(default)]
    pub approval_process_id: Option<ApprovalProcessId>,
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

    pub(super) fn conclude_customer_approval(&mut self, approved: bool) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityProposalEvent::CustomerApprovalConcluded { .. }
        );

        let (status, approval_process_id) = if approved {
            (
                CreditFacilityProposalStatus::PendingApproval,
                Some(self.id.into()),
            )
        } else {
            (CreditFacilityProposalStatus::CustomerDenied, None)
        };

        self.events
            .push(CreditFacilityProposalEvent::CustomerApprovalConcluded {
                status,
                approval_process_id,
            });

        self.approval_process_id = approval_process_id;

        Idempotent::Executed(())
    }

    pub(super) fn conclude_approval_process(
        &mut self,
        approved: bool,
    ) -> Idempotent<Option<(NewPendingCreditFacility, Option<CustodianId>)>> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityProposalEvent::ApprovalProcessConcluded { .. }
        );

        let approval_process_id = self
            .approval_process_id
            .expect("approval process id not set");

        let status = if approved {
            CreditFacilityProposalStatus::Approved
        } else {
            CreditFacilityProposalStatus::Denied
        };

        self.events
            .push(CreditFacilityProposalEvent::ApprovalProcessConcluded {
                approval_process_id,
                status,
            });
        if approved {
            let new_pending_facility = NewPendingCreditFacilityBuilder::default()
                .id(self.id)
                .credit_facility_proposal_id(self.id)
                .customer_id(self.customer_id)
                .customer_type(self.customer_type)
                .approval_process_id(approval_process_id)
                .ledger_tx_id(LedgerTxId::new())
                .account_ids(crate::PendingCreditFacilityAccountIds::new())
                .disbursal_credit_account_id(self.disbursal_credit_account_id)
                .collateral_id(CollateralId::new())
                .terms(self.terms)
                .amount(self.amount)
                .build()
                .expect("Could not build new pending credit facility");

            return Idempotent::Executed(Some((new_pending_facility, self.custodian_id)));
        }

        Idempotent::Executed(None)
    }

    pub fn status(&self) -> CreditFacilityProposalStatus {
        self.events
            .iter_all()
            .rev()
            .map(|event| match event {
                CreditFacilityProposalEvent::ApprovalProcessConcluded { status, .. } => *status,
                CreditFacilityProposalEvent::CustomerApprovalConcluded { status, .. } => *status,
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
                        .disbursal_credit_account_id(*disbursal_credit_account_id)
                        .terms(*terms)
                        .amount(*amount);
                }
                CreditFacilityProposalEvent::CustomerApprovalConcluded {
                    approval_process_id,
                    ..
                } => builder = builder.approval_process_id(*approval_process_id),
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
    #[builder(setter(into), default)]
    pub(super) custodian_id: Option<CustodianId>,
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
                disbursal_credit_account_id: self.disbursal_credit_account_id,
                terms: self.terms,
                amount: self.amount,
                status: CreditFacilityProposalStatus::PendingCustomerApproval,
            }],
        )
    }
}
