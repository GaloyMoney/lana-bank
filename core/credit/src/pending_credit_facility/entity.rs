use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{
    credit_facility::NewCreditFacilityBuilder,
    ledger::{
        CreditFacilityProposalBalanceSummary, PendingCreditFacilityAccountIds,
        PendingCreditFacilityCreation,
    },
    primitives::*,
    terms::TermValues,
};

use super::error::PendingCreditFacilityError;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PendingCreditFacilityId")]
pub enum PendingCreditFacilityEvent {
    Initialized {
        id: PendingCreditFacilityId,
        credit_facility_proposal_id: CreditFacilityProposalId,
        ledger_tx_id: LedgerTxId,
        approval_process_id: ApprovalProcessId,
        customer_id: CustomerId,
        customer_type: CustomerType,
        collateral_id: CollateralId,
        terms: TermValues,
        amount: UsdCents,
        account_ids: PendingCreditFacilityAccountIds,
        disbursal_credit_account_id: CalaAccountId,
    },
    CollateralizationStateChanged {
        collateralization_state: PendingCreditFacilityCollateralizationState,
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
pub struct PendingCreditFacility {
    pub id: PendingCreditFacilityId,
    pub ledger_tx_id: LedgerTxId,
    pub approval_process_id: ApprovalProcessId,
    pub account_ids: PendingCreditFacilityAccountIds,
    pub disbursal_credit_account_id: CalaAccountId,
    pub customer_id: CustomerId,
    pub customer_type: CustomerType,
    pub collateral_id: CollateralId,
    pub amount: UsdCents,
    pub terms: TermValues,

    events: EntityEvents<PendingCreditFacilityEvent>,
}

impl PendingCreditFacility {
    pub fn creation_data(&self) -> PendingCreditFacilityCreation {
        match self.events.iter_all().next() {
            Some(PendingCreditFacilityEvent::Initialized {
                ledger_tx_id,
                account_ids,
                amount,
                ..
            }) => PendingCreditFacilityCreation {
                tx_id: *ledger_tx_id,
                tx_ref: format!("{}-create", self.id),
                pending_credit_facility_account_ids: *account_ids,
                facility_amount: *amount,
            },
            _ => unreachable!("Initialized event must be the first event"),
        }
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn status(&self) -> PendingCreditFacilityStatus {
        if self.is_completed() {
            return PendingCreditFacilityStatus::Completed;
        }

        PendingCreditFacilityStatus::PendingCollateralization
    }

    pub(crate) fn update_collateralization(
        &mut self,
        price: PriceOfOneBTC,
        balances: CreditFacilityProposalBalanceSummary,
    ) -> Idempotent<Option<PendingCreditFacilityCollateralizationState>> {
        if self.is_completed() {
            return Idempotent::Ignored;
        }

        let ratio_changed = self.update_collateralization_ratio(&balances).did_execute();

        let is_fully_collateralized =
            balances.facility_amount_cvl(price) >= self.terms.margin_call_cvl;

        let calculated_collateralization_state = if is_fully_collateralized {
            PendingCreditFacilityCollateralizationState::FullyCollateralized
        } else {
            PendingCreditFacilityCollateralizationState::UnderCollateralized
        };

        if calculated_collateralization_state != self.last_collateralization_state() {
            self.events
                .push(PendingCreditFacilityEvent::CollateralizationStateChanged {
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
            .push(PendingCreditFacilityEvent::CollateralizationRatioChanged {
                collateralization_ratio: ratio,
            });
        Idempotent::Executed(())
    }

    pub fn last_collateralization_ratio(&self) -> CollateralizationRatio {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                PendingCreditFacilityEvent::CollateralizationRatioChanged {
                    collateralization_ratio: ratio,
                    ..
                } => Some(*ratio),
                _ => None,
            })
            .unwrap_or(CollateralizationRatio::default())
    }

    pub fn last_collateralization_state(&self) -> PendingCreditFacilityCollateralizationState {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                PendingCreditFacilityEvent::CollateralizationStateChanged {
                    collateralization_state,
                    ..
                } => Some(*collateralization_state),
                _ => None,
            })
            .unwrap_or(PendingCreditFacilityCollateralizationState::UnderCollateralized)
    }

    pub(super) fn complete(
        &mut self,
        balances: CreditFacilityProposalBalanceSummary,
        price: PriceOfOneBTC,
        time: DateTime<Utc>,
    ) -> Result<Idempotent<NewCreditFacilityBuilder>, PendingCreditFacilityError> {
        idempotency_guard!(
            self.events.iter_all(),
            PendingCreditFacilityEvent::Completed { .. }
        );

        if !self.terms.is_proposal_completion_allowed(balances, price) {
            return Err(PendingCreditFacilityError::BelowMarginLimit);
        }

        self.events.push(PendingCreditFacilityEvent::Completed {});

        let mut new_credit_facility = NewCreditFacilityBuilder::default();
        new_credit_facility
            .id(self.id)
            .pending_credit_facility_id(self.id)
            .ledger_tx_id(LedgerTxId::new())
            .customer_id(self.customer_id)
            .customer_type(self.customer_type)
            .account_ids(crate::CreditFacilityLedgerAccountIds::from(
                self.account_ids,
            ))
            .disbursal_credit_account_id(self.disbursal_credit_account_id)
            .collateral_id(self.collateral_id)
            .terms(self.terms)
            .amount(self.amount)
            .activated_at(crate::time::now())
            .maturity_date(self.terms.maturity_date(time));

        Ok(Idempotent::Executed(new_credit_facility))
    }

    fn is_completed(&self) -> bool {
        self.events
            .iter_all()
            .any(|event| matches!(event, PendingCreditFacilityEvent::Completed { .. }))
    }
}

impl TryFromEvents<PendingCreditFacilityEvent> for PendingCreditFacility {
    fn try_from_events(
        events: EntityEvents<PendingCreditFacilityEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = PendingCreditFacilityBuilder::default();
        for event in events.iter_all() {
            match event {
                PendingCreditFacilityEvent::Initialized {
                    id,
                    ledger_tx_id,
                    approval_process_id,
                    customer_id,
                    customer_type,
                    collateral_id,
                    amount,
                    account_ids,
                    disbursal_credit_account_id,
                    terms,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .approval_process_id(*approval_process_id)
                        .customer_id(*customer_id)
                        .customer_type(*customer_type)
                        .ledger_tx_id(*ledger_tx_id)
                        .collateral_id(*collateral_id)
                        .amount(*amount)
                        .terms(*terms)
                        .account_ids(*account_ids)
                        .disbursal_credit_account_id(*disbursal_credit_account_id)
                }
                PendingCreditFacilityEvent::CollateralizationStateChanged { .. } => {}
                PendingCreditFacilityEvent::CollateralizationRatioChanged { .. } => {}
                PendingCreditFacilityEvent::Completed { .. } => {}
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewPendingCreditFacility {
    #[builder(setter(into))]
    pub(super) id: PendingCreditFacilityId,
    #[builder(setter(into))]
    pub(super) credit_facility_proposal_id: CreditFacilityProposalId,
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
    pub(super) collateralization_state: PendingCreditFacilityCollateralizationState,
    pub(super) account_ids: PendingCreditFacilityAccountIds,
    disbursal_credit_account_id: CalaAccountId,
    terms: TermValues,
    amount: UsdCents,
}

impl NewPendingCreditFacility {
    pub fn builder() -> NewPendingCreditFacilityBuilder {
        NewPendingCreditFacilityBuilder::default()
    }
}

impl IntoEvents<PendingCreditFacilityEvent> for NewPendingCreditFacility {
    fn into_events(self) -> EntityEvents<PendingCreditFacilityEvent> {
        EntityEvents::init(
            self.id,
            [PendingCreditFacilityEvent::Initialized {
                id: self.id,
                credit_facility_proposal_id: self.credit_facility_proposal_id,
                ledger_tx_id: self.ledger_tx_id,
                approval_process_id: self.approval_process_id,
                customer_id: self.customer_id,
                customer_type: self.customer_type,
                collateral_id: self.collateral_id,
                terms: self.terms,
                amount: self.amount,
                account_ids: self.account_ids,
                disbursal_credit_account_id: self.disbursal_credit_account_id,
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

    fn initial_events() -> Vec<PendingCreditFacilityEvent> {
        vec![PendingCreditFacilityEvent::Initialized {
            id: PendingCreditFacilityId::new(),
            credit_facility_proposal_id: CreditFacilityProposalId::new(),
            ledger_tx_id: LedgerTxId::new(),
            approval_process_id: ApprovalProcessId::new(),
            customer_id: CustomerId::new(),
            customer_type: CustomerType::Individual,
            collateral_id: CollateralId::new(),
            amount: default_facility(),
            terms: default_terms(),
            account_ids: PendingCreditFacilityAccountIds::new(),
            disbursal_credit_account_id: CalaAccountId::new(),
        }]
    }

    fn proposal_from(events: Vec<PendingCreditFacilityEvent>) -> PendingCreditFacility {
        PendingCreditFacility::try_from_events(EntityEvents::init(
            PendingCreditFacilityId::new(),
            events,
        ))
        .unwrap()
    }

    mod complete {
        use super::*;

        #[test]
        fn errors_if_no_collateral() {
            let events = initial_events();
            let mut facility_proposal = proposal_from(events);

            assert!(matches!(
                facility_proposal.complete(default_balances(), default_price(), crate::time::now()),
                Err(PendingCreditFacilityError::BelowMarginLimit)
            ));
        }

        #[test]
        fn errors_if_collateral_below_margin() {
            let events = initial_events();
            let mut facility_proposal = proposal_from(events);

            assert!(matches!(
                facility_proposal.complete(
                    CreditFacilityProposalBalanceSummary::new(
                        default_facility(),
                        Satoshis::from(1_000)
                    ),
                    default_price(),
                    crate::time::now()
                ),
                Err(PendingCreditFacilityError::BelowMarginLimit)
            ));
        }

        #[test]
        fn ignored_if_already_completed() {
            let mut events = initial_events();
            events.extend([PendingCreditFacilityEvent::Completed {}]);
            let mut facility_proposal = proposal_from(events);

            assert!(matches!(
                facility_proposal.complete(default_balances(), default_price(), crate::time::now()),
                Ok(Idempotent::Ignored)
            ));
        }

        #[test]
        fn can_activate() {
            let events = initial_events();
            let mut facility_proposal = proposal_from(events);

            assert!(
                facility_proposal
                    .complete(
                        CreditFacilityProposalBalanceSummary::new(
                            default_facility(),
                            Satoshis::from(1_000_000)
                        ),
                        default_price(),
                        crate::time::now()
                    )
                    .is_ok()
            );
        }
    }
}
