use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{
    ledger::{
        CreditFacilityProposalAccountIds, CreditFacilityProposalBalanceSummary,
        CreditFacilityProposalCreation,
    },
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
        ledger_tx_id: LedgerTxId,
        customer_id: CustomerId,
        collateral_id: CollateralId,
        terms: TermValues,
        amount: UsdCents,
        account_ids: CreditFacilityProposalAccountIds,
        approval_process_id: ApprovalProcessId,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        approved: bool,
    },
    CollateralizationStateChanged {
        collateralization_state: CreditFacilityProposalCollateralizationState,
        collateral: Satoshis,
        price: PriceOfOneBTC,
    },
    CollateralizationRatioChanged {
        collateralization_ratio: CollateralizationRatio,
    },
    Completed {
        approved: bool,
    },
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
    pub terms: TermValues,

    events: EntityEvents<CreditFacilityProposalEvent>,
}

impl CreditFacilityProposal {
    pub fn creation_data(&self) -> CreditFacilityProposalCreation {
        match self.events.iter_all().next() {
            Some(CreditFacilityProposalEvent::Initialized {
                id,
                ledger_tx_id,
                account_ids,
                amount,
                ..
            }) => CreditFacilityProposalCreation {
                entity_id: *id,
                tx_id: *ledger_tx_id,
                tx_ref: format!("{}-create", self.id),
                credit_facility_proposal_account_ids: *account_ids,
                facility_amount: *amount,
            },
            _ => unreachable!("Initialized event must be the first event"),
        }
    }

    pub(crate) fn update_collateralization(
        &mut self,
        price: PriceOfOneBTC,
        balances: CreditFacilityProposalBalanceSummary,
    ) -> Idempotent<Option<CreditFacilityProposalCollateralizationState>> {
        let ratio_changed = self.update_collateralization_ratio(&balances).did_execute();

        let is_fully_collateralized =
            balances.facility_amount_cvl(price) >= self.terms.margin_call_cvl;

        let calculated_collateralization_state = if is_fully_collateralized {
            CreditFacilityProposalCollateralizationState::FullyCollateralized
        } else {
            CreditFacilityProposalCollateralizationState::UnderCollateralized
        };

        if calculated_collateralization_state != self.last_collateralization_state() {
            self.events
                .push(CreditFacilityProposalEvent::CollateralizationStateChanged {
                    collateralization_state: calculated_collateralization_state,
                    collateral: balances.collateral(),
                    price,
                });
            Idempotent::Executed(Some(calculated_collateralization_state))
        } else if ratio_changed {
            Idempotent::Executed(None)
        } else {
            Idempotent::Ignored
        }
    }

    fn update_collateralization_ratio(
        &mut self,
        balance: &CreditFacilityProposalBalanceSummary,
    ) -> Idempotent<()> {
        let ratio = balance.current_collateralization_ratio();

        if self.last_collateralization_ratio() == ratio {
            return Idempotent::Ignored;
        }

        self.events
            .push(CreditFacilityProposalEvent::CollateralizationRatioChanged {
                collateralization_ratio: ratio,
            });

        Idempotent::Executed(())
    }

    pub fn last_collateralization_ratio(&self) -> CollateralizationRatio {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityProposalEvent::CollateralizationRatioChanged {
                    collateralization_ratio: ratio,
                    ..
                } => Some(*ratio),
                _ => None,
            })
            .unwrap_or(CollateralizationRatio::default())
    }

    pub fn last_collateralization_state(&self) -> CreditFacilityProposalCollateralizationState {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityProposalEvent::CollateralizationStateChanged {
                    collateralization_state,
                    ..
                } => Some(*collateralization_state),
                _ => None,
            })
            .unwrap_or(CreditFacilityProposalCollateralizationState::UnderCollateralized)
    }

    pub(crate) fn is_approval_process_concluded(&self) -> bool {
        self.events.iter_all().any(|event| {
            matches!(
                event,
                CreditFacilityProposalEvent::ApprovalProcessConcluded { .. }
            )
        })
    }

    pub(crate) fn approval_process_concluded(&mut self, approved: bool) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityProposalEvent::ApprovalProcessConcluded { .. }
        );
        self.events
            .push(CreditFacilityProposalEvent::ApprovalProcessConcluded {
                approval_process_id: self.id.into(),
                approved,
            });
        Idempotent::Executed(())
    }

    fn _complete(&mut self, approved: bool) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityProposalEvent::Completed { .. }
        );
        self.events
            .push(CreditFacilityProposalEvent::Completed { approved });
        Idempotent::Executed(())
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
                    terms,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .customer_id(*customer_id)
                        .collateral_id(*collateral_id)
                        .amount(*amount)
                        .terms(*terms)
                        .account_ids(*account_ids)
                        .approval_process_id(*approval_process_id);
                }
                CreditFacilityProposalEvent::ApprovalProcessConcluded { .. } => {}
                CreditFacilityProposalEvent::CollateralizationStateChanged { .. } => {}
                CreditFacilityProposalEvent::CollateralizationRatioChanged { .. } => {}
                CreditFacilityProposalEvent::Completed { .. } => {}
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
            }],
        )
    }
}
