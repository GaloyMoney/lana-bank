use chrono::{DateTime, Datelike, Utc};
use derive_builder::Builder;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    entity::*,
    ledger::{
        customer::CustomerLedgerAccountIds,
        loan::{LoanAccountIds, LoanCollateralUpdate, LoanPaymentAmounts, LoanRepayment},
    },
    primitives::*,
};

use super::{error::LoanError, terms::TermValues, CVLPct, LoanApproval, LoanInterestAccrual};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoanReceivable {
    pub principal: UsdCents,
    pub interest: UsdCents,
}

pub enum LoanHistory {
    Payment(IncrementalPayment),
    Interest(InterestAccrued),
    Collateral(CollateralUpdated),
    Origination(LoanOrigination),
    Collateralization(CollateralizationUpdated),
}

pub struct IncrementalPayment {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub tx_id: LedgerTxId,
}

pub struct InterestAccrued {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub tx_id: LedgerTxId,
}

pub struct CollateralUpdated {
    pub satoshis: Satoshis,
    pub recorded_at: DateTime<Utc>,
    pub action: CollateralAction,
    pub tx_id: LedgerTxId,
}

pub struct LoanOrigination {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub tx_id: LedgerTxId,
}

pub struct CollateralizationUpdated {
    pub state: LoanCollaterizationState,
    pub collateral: Satoshis,
    pub outstanding_interest: UsdCents,
    pub outstanding_principal: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub price: PriceOfOneBTC,
}

impl LoanReceivable {
    pub fn total(&self) -> UsdCents {
        self.interest + self.principal
    }

    fn allocate_payment(&self, amount: UsdCents) -> Result<LoanPaymentAmounts, LoanError> {
        let mut remaining = amount;

        let interest = std::cmp::min(amount, self.interest);
        remaining -= interest;

        let principal = std::cmp::min(remaining, self.principal);
        remaining -= principal;

        Ok(LoanPaymentAmounts {
            interest,
            principal,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, async_graphql::Enum)]
pub enum LoanCollaterizationState {
    FullyCollateralized,
    UnderMarginCallThreshold,
    UnderLiquidationThreshold,
    NoCollateral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LoanEvent {
    Initialized {
        id: LoanId,
        customer_id: CustomerId,
        principal: UsdCents,
        terms: TermValues,
        account_ids: LoanAccountIds,
        customer_account_ids: CustomerLedgerAccountIds,
        audit_id: AuditEntryId,
    },
    CollateralUpdated {
        tx_id: LedgerTxId,
        tx_ref: String,
        total_collateral: Satoshis,
        abs_diff: Satoshis,
        action: CollateralAction,
        recorded_at: DateTime<Utc>,
        audit_id: AuditEntryId,
    },
    CollateralizationChanged {
        state: LoanCollaterizationState,
        collateral: Satoshis,
        outstanding: LoanReceivable,
        price: PriceOfOneBTC,
        recorded_at: DateTime<Utc>,
        audit_id: AuditEntryId,
    },
    Approved {
        tx_id: LedgerTxId,
        audit_id: AuditEntryId,
        recorded_at: DateTime<Utc>,
    },
    InterestIncurred {
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
        recorded_at: DateTime<Utc>,
        audit_id: AuditEntryId,
    },
    PaymentRecorded {
        tx_id: LedgerTxId,
        tx_ref: String,
        principal_amount: UsdCents,
        interest_amount: UsdCents,
        recorded_at: DateTime<Utc>,
        audit_id: AuditEntryId,
    },
    Completed {
        completed_at: DateTime<Utc>,
        audit_id: AuditEntryId,
    },
}

impl EntityEvent for LoanEvent {
    type EntityId = LoanId;
    fn event_table_name() -> &'static str {
        "loan_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Loan {
    pub id: LoanId,
    pub customer_id: CustomerId,
    pub terms: TermValues,
    pub account_ids: LoanAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
    pub(super) events: EntityEvents<LoanEvent>,
}

impl Loan {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at
            .expect("entity_first_persisted_at not found")
    }

    pub fn initial_principal(&self) -> UsdCents {
        if let Some(LoanEvent::Initialized { principal, .. }) = self.events.iter().next() {
            *principal
        } else {
            unreachable!("Initialized event not found")
        }
    }

    fn principal_payments(&self) -> UsdCents {
        self.events
            .iter()
            .filter_map(|event| match event {
                LoanEvent::PaymentRecorded {
                    principal_amount, ..
                } => Some(*principal_amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
    }

    fn interest_payments(&self) -> UsdCents {
        self.events
            .iter()
            .filter_map(|event| match event {
                LoanEvent::PaymentRecorded {
                    interest_amount, ..
                } => Some(*interest_amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
    }

    fn interest_recorded(&self) -> UsdCents {
        self.events
            .iter()
            .filter_map(|event| match event {
                LoanEvent::InterestIncurred { amount, .. } => Some(*amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
    }

    pub fn outstanding(&self) -> LoanReceivable {
        LoanReceivable {
            principal: self.initial_principal() - self.principal_payments(),
            interest: self.interest_recorded() - self.interest_payments(),
        }
    }

    pub fn collateral(&self) -> Satoshis {
        self.events
            .iter()
            .rev()
            .find_map(|event| match event {
                LoanEvent::CollateralUpdated {
                    total_collateral, ..
                } => Some(*total_collateral),
                _ => None,
            })
            .unwrap_or(Satoshis::ZERO)
    }

    pub fn history(&self) -> Vec<LoanHistory> {
        let mut history = vec![];

        for event in self.events.iter().rev() {
            match event {
                LoanEvent::CollateralUpdated {
                    abs_diff,
                    action,
                    recorded_at,
                    tx_id,
                    ..
                } => match action {
                    CollateralAction::Add => {
                        history.push(LoanHistory::Collateral(CollateralUpdated {
                            satoshis: *abs_diff,
                            action: *action,
                            recorded_at: *recorded_at,
                            tx_id: *tx_id,
                        }));
                    }
                    CollateralAction::Remove => {
                        history.push(LoanHistory::Collateral(CollateralUpdated {
                            satoshis: *abs_diff,
                            action: *action,
                            recorded_at: *recorded_at,
                            tx_id: *tx_id,
                        }));
                    }
                },

                LoanEvent::InterestIncurred {
                    amount,
                    recorded_at,
                    tx_id,
                    ..
                } => {
                    history.push(LoanHistory::Interest(InterestAccrued {
                        cents: *amount,
                        recorded_at: *recorded_at,
                        tx_id: *tx_id,
                    }));
                }

                LoanEvent::PaymentRecorded {
                    principal_amount,
                    interest_amount,
                    recorded_at: transaction_recorded_at,
                    tx_id,
                    ..
                } => {
                    history.push(LoanHistory::Payment(IncrementalPayment {
                        cents: *principal_amount + *interest_amount,
                        recorded_at: *transaction_recorded_at,
                        tx_id: *tx_id,
                    }));
                }

                LoanEvent::Approved {
                    tx_id, recorded_at, ..
                } => {
                    history.push(LoanHistory::Origination(LoanOrigination {
                        cents: self.initial_principal(),
                        recorded_at: *recorded_at,
                        tx_id: *tx_id,
                    }));
                }

                LoanEvent::CollateralizationChanged {
                    state,
                    collateral,
                    outstanding,
                    price,
                    recorded_at,
                    ..
                } => {
                    history.push(LoanHistory::Collateralization(CollateralizationUpdated {
                        state: *state,
                        collateral: *collateral,
                        outstanding_interest: outstanding.interest,
                        outstanding_principal: outstanding.principal,
                        price: *price,
                        recorded_at: *recorded_at,
                    }));
                }
                _ => {}
            }
        }

        history
    }

    pub(super) fn is_approved(&self) -> bool {
        for event in self.events.iter() {
            match event {
                LoanEvent::Approved { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    pub fn status(&self) -> LoanStatus {
        if self.is_completed() {
            LoanStatus::Closed
        } else if self.is_approved() {
            LoanStatus::Active
        } else {
            LoanStatus::New
        }
    }

    pub fn collateralization(&self) -> LoanCollaterizationState {
        if self.status() == LoanStatus::Closed {
            return LoanCollaterizationState::NoCollateral;
        }

        self.events
            .iter()
            .rev()
            .find_map(|event| match event {
                LoanEvent::CollateralizationChanged { state, .. } => Some(*state),
                _ => None,
            })
            .unwrap_or(LoanCollaterizationState::NoCollateral)
    }

    pub(super) fn collateralization_ratio(&self) -> Option<Decimal> {
        let outstanding = Decimal::from(self.outstanding().total().into_inner());
        if outstanding > Decimal::ZERO {
            Some(rust_decimal::Decimal::from(self.collateral().into_inner()) / outstanding)
        } else {
            None
        }
    }

    pub(super) fn initiate_approval(&mut self) -> Result<LoanApproval, LoanError> {
        if self.is_approved() {
            return Err(LoanError::AlreadyApproved);
        }

        if self.collateral() == Satoshis::ZERO {
            return Err(LoanError::NoCollateral);
        }
        let tx_ref = format!("{}-approval", self.id);
        Ok(LoanApproval {
            initial_principal: self.initial_principal(),
            tx_ref,
            tx_id: LedgerTxId::new(),
            loan_account_ids: self.account_ids,
            customer_account_ids: self.customer_account_ids,
        })
    }

    pub(super) fn confirm_approval(
        &mut self,
        LoanApproval { tx_id, .. }: LoanApproval,
        executed_at: DateTime<Utc>,
        audit_id: AuditEntryId,
    ) {
        self.events.push(LoanEvent::Approved {
            tx_id,
            audit_id,
            recorded_at: executed_at,
        });
    }

    pub fn approved_at(&self) -> Option<DateTime<Utc>> {
        self.events.iter().find_map(|event| match event {
            LoanEvent::Approved { recorded_at, .. } => Some(*recorded_at),
            _ => None,
        })
    }

    pub fn next_interest_at(&self) -> Option<DateTime<Utc>> {
        if !self.is_completed() && !self.is_expired() {
            Some(
                self.terms
                    .interval
                    .next_interest_payment(chrono::Utc::now()),
            )
        } else {
            None
        }
    }

    fn is_expired(&self) -> bool {
        match self.expires_at() {
            Some(expiration_date) => Utc::now() > expiration_date,
            None => false,
        }
    }

    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.approved_at()
            .map(|a| self.terms.duration.expiration_date(a))
    }

    fn count_interest_incurred(&self) -> usize {
        self.events
            .iter()
            .filter(|event| matches!(event, LoanEvent::InterestIncurred { .. }))
            .count()
    }

    pub fn is_completed(&self) -> bool {
        self.events
            .iter()
            .any(|event| matches!(event, LoanEvent::Completed { .. }))
    }

    fn days_for_interest_calculation(&self) -> u32 {
        if self.count_interest_incurred() == 0 {
            self.terms
                .interval
                .next_interest_payment(self.created_at())
                .day()
                - self.created_at().day()
                + 1 // 1 is added to account for the day when the loan was
                    // approved
        } else {
            self.terms.interval.next_interest_payment(Utc::now()).day()
        }
    }

    pub fn initiate_interest(&self) -> Result<LoanInterestAccrual, LoanError> {
        if self.is_completed() {
            return Err(LoanError::AlreadyCompleted);
        }

        let interest = self.terms.calculate_interest(
            self.initial_principal(),
            self.days_for_interest_calculation(),
        );

        let tx_ref = format!(
            "{}-interest-{}",
            self.id,
            self.count_interest_incurred() + 1
        );
        Ok(LoanInterestAccrual {
            interest,
            tx_ref,
            tx_id: LedgerTxId::new(),
            loan_account_ids: self.account_ids,
        })
    }

    pub fn confirm_interest(
        &mut self,
        LoanInterestAccrual {
            interest,
            tx_ref,
            tx_id,
            ..
        }: LoanInterestAccrual,
        executed_at: DateTime<Utc>,
        audit_id: AuditEntryId,
    ) {
        self.events.push(LoanEvent::InterestIncurred {
            tx_id,
            tx_ref,
            amount: interest,
            recorded_at: executed_at,
            audit_id,
        });
    }

    pub(super) fn initiate_repayment(
        &self,
        record_amount: UsdCents,
    ) -> Result<LoanRepayment, LoanError> {
        if self.is_completed() {
            return Err(LoanError::AlreadyCompleted);
        }
        let outstanding = self.outstanding();
        if outstanding.total() < record_amount {
            return Err(LoanError::PaymentExceedsOutstandingLoanAmount(
                record_amount,
                outstanding.total(),
            ));
        }
        let amounts = outstanding.allocate_payment(record_amount)?;
        let tx_ref = format!("{}-payment-{}", self.id, self.count_recorded_payments() + 1);
        let res = if outstanding.total() == record_amount {
            let collateral_tx_ref = format!("{}-collateral", self.id,);
            LoanRepayment::Final {
                payment_tx_id: LedgerTxId::new(),
                payment_tx_ref: tx_ref,
                collateral_tx_id: LedgerTxId::new(),
                collateral_tx_ref,
                collateral: self.collateral(),
                loan_account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
                amounts,
            }
        } else {
            LoanRepayment::Partial {
                tx_id: LedgerTxId::new(),
                tx_ref,
                loan_account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
                amounts,
            }
        };
        Ok(res)
    }

    pub fn confirm_repayment(
        &mut self,
        repayment: LoanRepayment,
        recorded_at: DateTime<Utc>,
        audit_id: AuditEntryId,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
    ) {
        match repayment {
            LoanRepayment::Partial {
                tx_id,
                tx_ref,
                amounts:
                    LoanPaymentAmounts {
                        interest,
                        principal,
                    },
                ..
            } => {
                self.events.push(LoanEvent::PaymentRecorded {
                    tx_id,
                    tx_ref,
                    principal_amount: principal,
                    interest_amount: interest,
                    recorded_at,
                    audit_id,
                });
                self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, audit_id);
            }
            LoanRepayment::Final {
                payment_tx_id,
                payment_tx_ref,
                collateral_tx_id,
                collateral_tx_ref,
                amounts:
                    LoanPaymentAmounts {
                        interest,
                        principal,
                    },
                collateral,
                ..
            } => {
                self.events.push(LoanEvent::PaymentRecorded {
                    tx_id: payment_tx_id,
                    tx_ref: payment_tx_ref,
                    principal_amount: principal,
                    interest_amount: interest,
                    recorded_at,
                    audit_id,
                });
                self.confirm_collateral_update(
                    LoanCollateralUpdate {
                        loan_account_ids: self.account_ids,
                        tx_id: collateral_tx_id,
                        tx_ref: collateral_tx_ref.clone(),
                        abs_diff: collateral,
                        action: CollateralAction::Remove,
                    },
                    recorded_at,
                    audit_id,
                    price,
                    upgrade_buffer_cvl_pct,
                );
                self.events.push(LoanEvent::Completed {
                    completed_at: recorded_at,
                    audit_id,
                });
            }
        }
    }

    pub fn cvl(&self, price: PriceOfOneBTC) -> CVLPct {
        let collateral_value = price.sats_to_cents_round_down(self.collateral());

        if collateral_value == UsdCents::ZERO {
            CVLPct::ZERO
        } else {
            CVLPct::from_loan_amounts(collateral_value, self.outstanding().total())
        }
    }

    fn calculate_collaterization(&self, price: PriceOfOneBTC) -> LoanCollaterizationState {
        let margin_call_cvl = self.terms.margin_call_cvl;
        let liquidation_cvl = self.terms.liquidation_cvl;

        let current_cvl = self.cvl(price);
        if current_cvl == CVLPct::ZERO {
            LoanCollaterizationState::NoCollateral
        } else if current_cvl >= margin_call_cvl {
            LoanCollaterizationState::FullyCollateralized
        } else if current_cvl >= liquidation_cvl {
            LoanCollaterizationState::UnderMarginCallThreshold
        } else {
            LoanCollaterizationState::UnderLiquidationThreshold
        }
    }

    pub fn maybe_update_collateralization_with_liquidation_override(
        &mut self,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
        audit_id: AuditEntryId,
    ) -> Option<LoanCollaterizationState> {
        let current_collateralization = self.collateralization();
        let calculated_collateralization = &self.calculate_collaterization(price);

        if current_collateralization == LoanCollaterizationState::UnderLiquidationThreshold
            && current_collateralization != *calculated_collateralization
        {
            self.events.push(LoanEvent::CollateralizationChanged {
                state: *calculated_collateralization,
                collateral: self.collateral(),
                outstanding: self.outstanding(),
                price,
                recorded_at: Utc::now(),
                audit_id,
            });

            return Some(*calculated_collateralization);
        }

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, audit_id)
    }

    pub fn maybe_update_collateralization(
        &mut self,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
        audit_id: AuditEntryId,
    ) -> Option<LoanCollaterizationState> {
        let collateral = self.collateral();
        let current_collateralization = self.collateralization();
        let calculated_collateralization = &self.calculate_collaterization(price);

        match (
            self.status(),
            current_collateralization,
            *calculated_collateralization,
        ) {
            // Redundant same state changes
            (_, LoanCollaterizationState::NoCollateral, LoanCollaterizationState::NoCollateral)
            | (
                _,
                LoanCollaterizationState::FullyCollateralized,
                LoanCollaterizationState::FullyCollateralized,
            )
            | (
                _,
                LoanCollaterizationState::UnderMarginCallThreshold,
                LoanCollaterizationState::UnderMarginCallThreshold,
            )
            | (
                _,
                LoanCollaterizationState::UnderLiquidationThreshold,
                LoanCollaterizationState::UnderLiquidationThreshold,
            ) => None,

            // Invalid collateral liquidation changes
            (LoanStatus::Active, LoanCollaterizationState::UnderLiquidationThreshold, _) => None,

            // Valid buffered collateral upgraded change
            (
                LoanStatus::Active,
                LoanCollaterizationState::UnderMarginCallThreshold,
                LoanCollaterizationState::FullyCollateralized,
            ) => {
                let margin_call_cvl = self.terms.margin_call_cvl;
                let current_cvl = self.cvl(price);

                if margin_call_cvl.is_significantly_lower_than(current_cvl, upgrade_buffer_cvl_pct)
                {
                    self.events.push(LoanEvent::CollateralizationChanged {
                        state: *calculated_collateralization,
                        collateral,
                        outstanding: self.outstanding(),
                        price,
                        recorded_at: Utc::now(),
                        audit_id,
                    });

                    Some(*calculated_collateralization)
                } else {
                    None
                }
            }

            // Valid other collateral changes
            (_, LoanCollaterizationState::NoCollateral, _)
            | (_, LoanCollaterizationState::FullyCollateralized, _)
            | (_, LoanCollaterizationState::UnderMarginCallThreshold, _)
            | (_, LoanCollaterizationState::UnderLiquidationThreshold, _) => {
                self.events.push(LoanEvent::CollateralizationChanged {
                    state: *calculated_collateralization,
                    collateral,
                    outstanding: self.outstanding(),
                    price,
                    recorded_at: Utc::now(),
                    audit_id,
                });

                Some(*calculated_collateralization)
            }
        }
    }

    fn count_recorded_payments(&self) -> usize {
        self.events
            .iter()
            .filter(|event| matches!(event, LoanEvent::PaymentRecorded { .. }))
            .count()
    }

    fn count_collateral_adjustments(&self) -> usize {
        self.events
            .iter()
            .filter(|event| matches!(event, LoanEvent::CollateralUpdated { .. }))
            .count()
    }

    pub(super) fn initiate_collateral_update(
        &self,
        updated_collateral: Satoshis,
    ) -> Result<LoanCollateralUpdate, LoanError> {
        let current_collateral = self.collateral();
        let diff =
            SignedSatoshis::from(updated_collateral) - SignedSatoshis::from(current_collateral);

        if diff == SignedSatoshis::ZERO {
            return Err(LoanError::CollateralNotUpdated(
                current_collateral,
                updated_collateral,
            ));
        }

        let (collateral, action) = if diff > SignedSatoshis::ZERO {
            (Satoshis::try_from(diff)?, CollateralAction::Add)
        } else {
            (Satoshis::try_from(diff.abs())?, CollateralAction::Remove)
        };

        let tx_ref = format!(
            "{}-collateral-{}",
            self.id,
            self.count_collateral_adjustments() + 1
        );

        let tx_id = LedgerTxId::new();

        Ok(LoanCollateralUpdate {
            abs_diff: collateral,
            loan_account_ids: self.account_ids,
            tx_ref,
            tx_id,
            action,
        })
    }

    pub(super) fn confirm_collateral_update(
        &mut self,
        LoanCollateralUpdate {
            tx_id,
            tx_ref,
            abs_diff,
            action,
            ..
        }: LoanCollateralUpdate,
        executed_at: DateTime<Utc>,
        audit_id: AuditEntryId,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
    ) -> Option<LoanCollaterizationState> {
        let mut total_collateral = self.collateral();
        total_collateral = match action {
            CollateralAction::Add => total_collateral + abs_diff,
            CollateralAction::Remove => total_collateral - abs_diff,
        };
        self.events.push(LoanEvent::CollateralUpdated {
            tx_id,
            tx_ref,
            total_collateral,
            abs_diff,
            action,
            recorded_at: executed_at,
            audit_id,
        });

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, audit_id)
    }
}

impl Entity for Loan {
    type Event = LoanEvent;
}

impl TryFrom<EntityEvents<LoanEvent>> for Loan {
    type Error = EntityError;

    fn try_from(events: EntityEvents<LoanEvent>) -> Result<Self, Self::Error> {
        let mut builder = LoanBuilder::default();
        for event in events.iter() {
            match event {
                LoanEvent::Initialized {
                    id,
                    customer_id,
                    account_ids,
                    customer_account_ids,
                    terms,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .customer_id(*customer_id)
                        .terms(terms.clone())
                        .account_ids(*account_ids)
                        .customer_account_ids(*customer_account_ids)
                }
                LoanEvent::Approved { .. } => (),
                LoanEvent::CollateralizationChanged { .. } => (),
                LoanEvent::InterestIncurred { .. } => (),
                LoanEvent::PaymentRecorded { .. } => (),
                LoanEvent::Completed { .. } => (),
                LoanEvent::CollateralUpdated { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewLoan {
    #[builder(setter(into))]
    pub(super) id: LoanId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    terms: TermValues,
    principal: UsdCents,
    account_ids: LoanAccountIds,
    customer_account_ids: CustomerLedgerAccountIds,
    #[builder(setter(into))]
    pub(super) audit_id: AuditEntryId,
}

impl NewLoan {
    pub fn builder(audit_id: AuditEntryId) -> NewLoanBuilder {
        let mut builder = NewLoanBuilder::default();
        builder.audit_id(audit_id);
        builder
    }

    pub(super) fn initial_events(self) -> EntityEvents<LoanEvent> {
        EntityEvents::init(
            self.id,
            [LoanEvent::Initialized {
                id: self.id,
                customer_id: self.customer_id,
                principal: self.principal,
                terms: self.terms,
                account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
                audit_id: self.audit_id,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use crate::loan::{Duration, InterestInterval};

    use super::*;

    fn terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(3))
            .interval(InterestInterval::EndOfMonth)
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term")
    }

    fn dummy_audit_id() -> AuditEntryId {
        let i: i64 = 0;
        i.into()
    }

    fn init_events() -> EntityEvents<LoanEvent> {
        EntityEvents::init(
            LoanId::new(),
            [
                LoanEvent::Initialized {
                    id: LoanId::new(),
                    customer_id: CustomerId::new(),
                    principal: UsdCents::from(100),
                    terms: terms(),
                    account_ids: LoanAccountIds::new(),
                    customer_account_ids: CustomerLedgerAccountIds::new(),
                    audit_id: dummy_audit_id(),
                },
                LoanEvent::InterestIncurred {
                    tx_id: LedgerTxId::new(),
                    tx_ref: "tx_ref".to_string(),
                    amount: UsdCents::from(5),
                    recorded_at: Utc::now(),
                    audit_id: dummy_audit_id(),
                },
            ],
        )
    }

    fn default_price() -> PriceOfOneBTC {
        PriceOfOneBTC::new(UsdCents::from(5000000))
    }

    fn default_upgrade_buffer_cvl_pct() -> CVLPct {
        CVLPct::new(5)
    }

    #[test]
    fn outstanding() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        assert_eq!(
            loan.outstanding(),
            LoanReceivable {
                principal: UsdCents::from(100),
                interest: UsdCents::from(5)
            }
        );
        let amount = UsdCents::from(4);
        let repayment = loan.initiate_repayment(amount).unwrap();
        loan.confirm_repayment(
            repayment,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        assert_eq!(
            loan.outstanding(),
            LoanReceivable {
                principal: UsdCents::from(100),
                interest: UsdCents::from(1)
            }
        );
        let amount = UsdCents::from(2);
        let repayment = loan.initiate_repayment(amount).unwrap();
        loan.confirm_repayment(
            repayment,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        assert_eq!(
            loan.outstanding(),
            LoanReceivable {
                principal: UsdCents::from(99),
                interest: UsdCents::ZERO
            }
        );

        let amount = UsdCents::from(100);
        let res = loan.initiate_repayment(amount);
        assert!(res.is_err());

        let amount = UsdCents::from(99);
        let repayment = loan.initiate_repayment(amount).unwrap();
        loan.confirm_repayment(
            repayment,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        assert_eq!(
            loan.outstanding(),
            LoanReceivable {
                principal: UsdCents::ZERO,
                interest: UsdCents::ZERO
            }
        );

        assert!(loan.is_completed());
    }

    #[test]
    fn prevent_double_approve() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let loan_approval = loan.initiate_approval();
        assert!(loan_approval.is_ok());
        loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_id());

        let loan_approval = loan.initiate_approval();
        assert!(loan_approval.is_err())
    }

    #[test]
    fn check_approved_at() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        assert_eq!(loan.approved_at(), None);
        assert_eq!(loan.expires_at(), None);

        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );

        let approval_time = Utc::now();
        let loan_approval = loan.initiate_approval();
        assert!(loan_approval.is_ok());
        loan.confirm_approval(loan_approval.unwrap(), approval_time, dummy_audit_id());
        assert_eq!(loan.approved_at(), Some(approval_time));
        assert!(loan.expires_at().is_some())
    }

    #[test]
    fn status() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        assert_eq!(loan.status(), LoanStatus::New);
        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let loan_approval = loan.initiate_approval().unwrap();
        loan.confirm_approval(loan_approval, Utc::now(), dummy_audit_id());
        assert_eq!(loan.status(), LoanStatus::Active);
        let amount = UsdCents::from(105);
        let repayment = loan.initiate_repayment(amount).unwrap();
        loan.confirm_repayment(
            repayment,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        assert_eq!(loan.status(), LoanStatus::Closed);
    }

    #[test]
    fn collateral() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        // initially collateral should be 0
        assert_eq!(loan.collateral(), Satoshis::ZERO);

        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        assert_eq!(loan.collateral(), Satoshis::from(10000));

        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(5000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        assert_eq!(loan.collateral(), Satoshis::from(5000));
    }

    #[test]
    fn cannot_approve_if_loan_has_no_collateral() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let res = loan.initiate_approval();
        assert!(matches!(res, Err(LoanError::NoCollateral)));
    }

    #[test]
    fn test_collateralization() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        assert_eq!(
            loan.collateralization(),
            LoanCollaterizationState::NoCollateral
        );
        assert_eq!(loan.collateral(), Satoshis::ZERO);

        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(12000000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let loan_approval = loan.initiate_approval();
        loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_id());
        loan.maybe_update_collateralization(
            default_price(),
            default_upgrade_buffer_cvl_pct(),
            dummy_audit_id(),
        );
        assert_eq!(
            loan.collateralization(),
            LoanCollaterizationState::FullyCollateralized,
        );
        assert_eq!(loan.collateral(), Satoshis::from(12000000));
    }

    #[test]
    fn calculate_cvl() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(3000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_id(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let loan_approval = loan.initiate_approval();
        loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_id());

        let expected_cvl = CVLPct::from(dec!(142));
        let cvl = loan.cvl(PriceOfOneBTC::new(UsdCents::from(5000000)));
        assert_eq!(cvl, expected_cvl);

        let expected_cvl = CVLPct::from(dec!(100));
        let cvl = loan.cvl(PriceOfOneBTC::new(UsdCents::from(3500000)));
        assert_eq!(cvl, expected_cvl);
    }

    mod maybe_update_collateralization {

        use super::*;

        fn price_from(value: u64) -> PriceOfOneBTC {
            PriceOfOneBTC::new(UsdCents::from(value))
        }

        #[test]
        fn test_transitions_no_status_change() {
            let upgrade_buffer_cvl_pct = CVLPct::new(5);

            // NoCollateral
            let mut loan = Loan::try_from(init_events()).unwrap();
            assert_eq!(
                loan.collateralization(),
                LoanCollaterizationState::NoCollateral
            );
            assert_eq!(loan.collateral(), Satoshis::ZERO);
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(7500000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_id()
                ),
                None
            );

            // NoCollateral -> FullyCollateralized
            let loan_collateral_update = loan
                .initiate_collateral_update(Satoshis::from(3000))
                .unwrap();
            loan.confirm_collateral_update(
                loan_collateral_update,
                Utc::now(),
                dummy_audit_id(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::FullyCollateralized);

            let loan_approval = loan.initiate_approval();
            loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_id());
            assert_eq!(loan.status(), LoanStatus::Active);

            // FullyCollateralized -> None
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(7500000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_id()
                ),
                None
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::FullyCollateralized);

            // FullyCollateralized -> UnderMarginCallThreshold
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(4350000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_id()
                ),
                Some(LoanCollaterizationState::UnderMarginCallThreshold)
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::UnderMarginCallThreshold);

            // UnderMarginCallThreshold -> None (CVL above margin_call but below buffer)
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(4550000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_id()
                ),
                None
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::UnderMarginCallThreshold);

            // UnderMarginCallThreshold -> FullyCollateralized (CVL above margin_call and buffer)
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(4600000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_id()
                ),
                Some(LoanCollaterizationState::FullyCollateralized)
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::FullyCollateralized);

            // UnderMarginCallThreshold -> UnderLiquidationThreshold
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(3000000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_id()
                ),
                Some(LoanCollaterizationState::UnderLiquidationThreshold)
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::UnderLiquidationThreshold);

            // UnderLiquidationThreshold -> None (CVL above Collaterization requirement)
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(10000000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_id()
                ),
                None
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::UnderLiquidationThreshold);
        }

        #[test]
        fn test_transitions_with_status_change() {
            let upgrade_buffer_cvl_pct = CVLPct::new(5);

            // LoanStatus::New
            let mut loan = Loan::try_from(init_events()).unwrap();

            assert_eq!(loan.status(), LoanStatus::New);
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(6500000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_id()
                ),
                None
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::NoCollateral);

            let loan_collateral_update = loan
                .initiate_collateral_update(Satoshis::from(100))
                .unwrap();
            loan.confirm_collateral_update(
                loan_collateral_update,
                Utc::now(),
                dummy_audit_id(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::UnderLiquidationThreshold);

            let loan_collateral_update = loan
                .initiate_collateral_update(Satoshis::from(2900))
                .unwrap();
            loan.confirm_collateral_update(
                loan_collateral_update,
                Utc::now(),
                dummy_audit_id(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::FullyCollateralized);

            // LoanStatus::Active
            let loan_approval = loan.initiate_approval();
            loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_id());
            assert_eq!(loan.status(), LoanStatus::Active);

            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(4350000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_id()
                ),
                Some(LoanCollaterizationState::UnderMarginCallThreshold)
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::UnderMarginCallThreshold);

            // LoanStatus::Closed
            assert_eq!(loan.status(), LoanStatus::Active);
            let repayment = loan.initiate_repayment(loan.outstanding().total()).unwrap();
            loan.confirm_repayment(
                repayment,
                Utc::now(),
                dummy_audit_id(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            assert_eq!(loan.status(), LoanStatus::Closed);
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::NoCollateral);

            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(10000000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_id()
                ),
                None
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::NoCollateral);
        }

        #[test]
        fn test_liquidation_override() {
            // Setup initial state
            let mut loan = Loan::try_from(init_events()).unwrap();
            let loan_collateral_update = loan
                .initiate_collateral_update(Satoshis::from(100))
                .unwrap();
            loan.confirm_collateral_update(
                loan_collateral_update,
                Utc::now(),
                dummy_audit_id(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::UnderLiquidationThreshold);

            let loan_approval = loan.initiate_approval();
            loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_id());
            assert_eq!(loan.status(), LoanStatus::Active);

            // Check allowed changes from Liquidation state
            let loan_collateral_update = loan
                .initiate_collateral_update(Satoshis::from(2900))
                .unwrap();
            loan.confirm_collateral_update(
                loan_collateral_update,
                Utc::now(),
                dummy_audit_id(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::UnderLiquidationThreshold);

            let price = price_from(100_000_00);
            assert_eq!(
                loan.maybe_update_collateralization(
                    price,
                    default_upgrade_buffer_cvl_pct(),
                    dummy_audit_id()
                ),
                None
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::UnderLiquidationThreshold);

            assert_eq!(
                loan.maybe_update_collateralization_with_liquidation_override(
                    price,
                    default_upgrade_buffer_cvl_pct(),
                    dummy_audit_id()
                ),
                Some(LoanCollaterizationState::FullyCollateralized)
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::FullyCollateralized);
        }
    }
}
