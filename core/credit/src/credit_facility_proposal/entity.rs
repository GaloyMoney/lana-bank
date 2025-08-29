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

use super::error::CreditFacilityProposalError;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CreditFacilityProposalId")]
pub enum CreditFacilityProposalEvent {
    Initialized {
        id: CreditFacilityProposalId,
        ledger_tx_id: LedgerTxId,
        customer_id: CustomerId,
        customer_type: CustomerType,
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
    Completed {},
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct CreditFacilityProposal {
    pub id: CreditFacilityProposalId,
    pub ledger_tx_id: LedgerTxId,
    pub approval_process_id: ApprovalProcessId,
    pub account_ids: CreditFacilityProposalAccountIds,
    pub customer_id: CustomerId,
    pub customer_type: CustomerType,
    pub collateral_id: CollateralId,
    pub amount: UsdCents,
    pub terms: TermValues,

    events: EntityEvents<CreditFacilityProposalEvent>,
}

impl CreditFacilityProposal {
    pub fn creation_data(&self) -> CreditFacilityProposalCreation {
        match self.events.iter_all().next() {
            Some(CreditFacilityProposalEvent::Initialized {
                ledger_tx_id,
                account_ids,
                amount,
                ..
            }) => CreditFacilityProposalCreation {
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

    pub(super) fn complete(
        &mut self,
        balances: CreditFacilityProposalBalanceSummary,
        price: PriceOfOneBTC,
    ) -> Result<Idempotent<()>, CreditFacilityProposalError> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityProposalEvent::Completed { .. }
        );

        if !self.is_approval_process_concluded() {
            return Err(CreditFacilityProposalError::ApprovalInProgress);
        }

        if !self.terms.is_proposal_completion_allowed(balances, price) {
            return Err(CreditFacilityProposalError::BelowMarginLimit);
        }

        self.events.push(CreditFacilityProposalEvent::Completed {});

        Ok(Idempotent::Executed(()))
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
                    ledger_tx_id,
                    customer_id,
                    customer_type,
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
                        .customer_type(*customer_type)
                        .ledger_tx_id(*ledger_tx_id)
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
    pub(super) customer_type: CustomerType,
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
                customer_type: self.customer_type,
                collateral_id: self.collateral_id,
                terms: self.terms,
                amount: self.amount,
                account_ids: self.account_ids,
                approval_process_id: self.approval_process_id,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use crate::{
        ObligationDuration,
        terms::{FacilityDuration, InterestInterval, OneTimeFeeRatePct},
    };

    use super::*;
    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(FacilityDuration::Months(3))
            .interest_due_duration_from_accrual(ObligationDuration::Days(0))
            .obligation_overdue_duration_from_due(None)
            .obligation_liquidation_duration_from_due(None)
            .accrual_cycle_interval(InterestInterval::EndOfMonth)
            .accrual_interval(InterestInterval::EndOfDay)
            .one_time_fee_rate(OneTimeFeeRatePct::new(5))
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term")
    }

    fn default_facility() -> UsdCents {
        UsdCents::from(10_00)
    }

    fn default_price() -> PriceOfOneBTC {
        PriceOfOneBTC::new(UsdCents::try_from_usd(dec!(100_000)).unwrap())
    }

    fn default_balances() -> CreditFacilityProposalBalanceSummary {
        CreditFacilityProposalBalanceSummary::new(default_facility(), Satoshis::ZERO)
    }

    fn initial_events() -> Vec<CreditFacilityProposalEvent> {
        vec![CreditFacilityProposalEvent::Initialized {
            id: CreditFacilityProposalId::new(),
            ledger_tx_id: LedgerTxId::new(),
            customer_id: CustomerId::new(),
            customer_type: CustomerType::Individual,
            collateral_id: CollateralId::new(),
            amount: default_facility(),
            terms: default_terms(),
            account_ids: CreditFacilityProposalAccountIds::new(),
            approval_process_id: ApprovalProcessId::new(),
        }]
    }

    fn proposal_from(events: Vec<CreditFacilityProposalEvent>) -> CreditFacilityProposal {
        CreditFacilityProposal::try_from_events(EntityEvents::init(
            CreditFacilityProposalId::new(),
            events,
        ))
        .unwrap()
    }

    mod complete {
        use super::*;
        #[test]
        fn errors_when_not_approved_yet() {
            let mut facility_proposal = proposal_from(initial_events());
            assert!(matches!(
                facility_proposal.complete(default_balances(), default_price()),
                Err(CreditFacilityProposalError::ApprovalInProgress)
            ));
        }

        // TODO: what happens when we deny a proposal and then try to complete it?

        //         #[test]
        //         fn errors_if_denied() {
        //             let mut events = initial_events();
        //             events.push(CreditFacilityEvent::ApprovalProcessConcluded {
        //                 approval_process_id: ApprovalProcessId::new(),
        //                 approved: false,
        //                 audit_info: dummy_audit_info(),
        //             });
        //             let mut credit_facility = facility_from(events);

        //             assert!(matches!(
        //                 credit_facility.activate(
        //                     Utc::now(),
        //                     default_price(),
        //                     default_balances(credit_facility.amount),
        //                     dummy_audit_info()
        //                 ),
        //                 Err(CreditFacilityError::Denied)
        //             ));
        //         }

        #[test]
        fn errors_if_no_collateral() {
            let mut events = initial_events();
            events.extend([CreditFacilityProposalEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: true,
            }]);
            let mut facility_proposal = proposal_from(events);

            assert!(matches!(
                facility_proposal.complete(default_balances(), default_price()),
                Err(CreditFacilityProposalError::BelowMarginLimit)
            ));
        }

        #[test]
        fn errors_if_collateral_below_margin() {
            let mut events = initial_events();
            events.extend([CreditFacilityProposalEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: true,
            }]);
            let mut facility_proposal = proposal_from(events);

            assert!(matches!(
                facility_proposal.complete(
                    CreditFacilityProposalBalanceSummary::new(
                        default_facility(),
                        Satoshis::from(1_000)
                    ),
                    default_price()
                ),
                Err(CreditFacilityProposalError::BelowMarginLimit)
            ));
        }

        #[test]
        fn ignored_if_already_completed() {
            let mut events = initial_events();
            events.extend([
                CreditFacilityProposalEvent::ApprovalProcessConcluded {
                    approval_process_id: ApprovalProcessId::new(),
                    approved: true,
                },
                CreditFacilityProposalEvent::Completed {},
            ]);
            let mut facility_proposal = proposal_from(events);

            assert!(matches!(
                facility_proposal.complete(default_balances(), default_price()),
                Ok(Idempotent::Ignored)
            ));
        }

        #[test]
        fn can_activate() {
            let mut events = initial_events();
            events.extend([CreditFacilityProposalEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: true,
            }]);
            let mut facility_proposal = proposal_from(events);

            assert!(
                facility_proposal
                    .complete(
                        CreditFacilityProposalBalanceSummary::new(
                            default_facility(),
                            Satoshis::from(1_000_000)
                        ),
                        default_price()
                    )
                    .is_ok()
            );
        }
    }
}
