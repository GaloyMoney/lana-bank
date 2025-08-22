use chrono::{DateTime, Utc};
use derive_builder::Builder;
use rust_decimal::Decimal;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::{
    ledger::{CreditFacilityProposalAccountIds, CreditFacilityProposalCreation},
    primitives::*,
    terms::TermValues,
};

#[allow(clippy::large_enum_variant)]
#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CreditFacilityProposalId")]
pub enum CreditFacilityProposalEvent {
    Initialized {
        id: CreditFacilityProposalId,
        ledger_tx_id: LedgerTxId,
        customer_id: CustomerId,
        collateral_id: CollateralId,
        terms: TermValues,
        amount: UsdCents,
        account_ids: CreditFacilityProposalAccountIds,
        approval_process_id: ApprovalProcessId,
        audit_info: AuditInfo,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        approved: bool,
        audit_info: AuditInfo,
    },

    CollateralizationStateChanged {
        is_collateralized: bool,
        collateral: Satoshis,
        price: PriceOfOneBTC,
        audit_info: AuditInfo,
    },
    CollateralizationRatioChanged {
        collateralization_ratio: Option<Decimal>,
        audit_info: AuditInfo,
    },
    Completed {},
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct CreditFacilityProposal {
    pub id: CreditFacilityProposalId,
    pub approval_process_id: ApprovalProcessId,
    pub account_ids: CreditFacilityProposalAccountIds,
    pub customer_id: CustomerId,
    pub collateral_id: CollateralId,
    pub amount: UsdCents,

    events: EntityEvents<CreditFacilityProposalEvent>,
}

impl CreditFacilityProposal {
    pub fn creation_data(&self) -> CreditFacilityProposalCreation {
        self.events
            .iter_all()
            .find_map(|event| {
                if let CreditFacilityProposalEvent::Initialized {
                    ledger_tx_id,
                    account_ids,
                    amount,
                    ..
                } = event
                {
                    Some(CreditFacilityProposalCreation {
                        tx_id: *ledger_tx_id,
                        tx_ref: format!("{}-create", self.id),
                        credit_facility_proposal_account_ids: *account_ids,
                        facility_amount: *amount,
                    })
                } else {
                    None
                }
            })
            .expect("Initialized event must be present")
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
                    collateral_id,
                    amount,
                    approval_process_id,
                    account_ids,
                    ..
                } => {
                    builder = builder
                        .id(id.clone())
                        .customer_id(customer_id.clone())
                        .collateral_id(collateral_id.clone())
                        .amount(*amount)
                        .account_ids(account_ids.clone())
                        .approval_process_id(approval_process_id.clone());
                }
                CreditFacilityProposalEvent::ApprovalProcessConcluded { .. } => {}
                CreditFacilityProposalEvent::CollateralizationStateChanged { .. } => {}
                CreditFacilityProposalEvent::CollateralizationRatioChanged { .. } => {}
                CreditFacilityProposalEvent::Completed {} => {}
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
    pub(super) ledger_tx_id: LedgerTxId,
    #[builder(setter(into))]
    pub(super) approval_process_id: ApprovalProcessId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    #[builder(setter(into))]
    pub(super) collateral_id: CollateralId,
    #[builder(setter(skip), default)]
    pub(super) collateralization_state: CreditFacilityProposalCollateralizationState,
    account_ids: CreditFacilityProposalAccountIds,
    terms: TermValues,

    amount: UsdCents,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
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
                ledger_tx_id: self.ledger_tx_id,
                customer_id: self.customer_id,
                collateral_id: self.collateral_id,
                terms: self.terms,
                amount: self.amount,
                account_ids: self.account_ids,
                approval_process_id: self.approval_process_id,
                audit_info: self.audit_info.clone(),
            }],
        )
    }
}
