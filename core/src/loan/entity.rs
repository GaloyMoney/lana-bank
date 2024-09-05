use chrono::{DateTime, Utc};
use derive_builder::Builder;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use crate::{
    entity::*,
    ledger::{
        customer::CustomerLedgerAccountIds,
        loan::{LoanAccountIds, LoanCollateralUpdate, LoanPaymentAmounts, LoanRepayment},
    },
    primitives::*,
};

use super::{
    error::LoanError, terms::TermValues, CVLPct, InterestPeriod, InterestPeriodEndDate,
    InterestPeriodStartDate, LoanApprovalData, LoanInterestAccrual,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoanReceivable {
    pub principal: UsdCents,
    pub interest: UsdCents,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoanRepaymentInPlan {
    Interest(RepaymentInPlan),
    Principal(RepaymentInPlan),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepaymentStatus {
    Upcoming,
    Due,
    Overdue,
    Paid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepaymentInPlan {
    pub status: RepaymentStatus,
    pub initial: UsdCents,
    pub outstanding: UsdCents,
    pub accrual_at: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
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

pub struct LoanApproval {
    pub user_id: UserId,
    pub approved_at: DateTime<Utc>,
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
        audit_info: AuditInfo,
    },
    CollateralUpdated {
        tx_id: LedgerTxId,
        tx_ref: String,
        total_collateral: Satoshis,
        abs_diff: Satoshis,
        action: CollateralAction,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    CollateralizationChanged {
        state: LoanCollaterizationState,
        collateral: Satoshis,
        outstanding: LoanReceivable,
        price: PriceOfOneBTC,
        recorded_at: DateTime<Utc>,
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
    InterestIncurred {
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    PaymentRecorded {
        tx_id: LedgerTxId,
        tx_ref: String,
        principal_amount: UsdCents,
        interest_amount: UsdCents,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    Completed {
        completed_at: DateTime<Utc>,
        audit_info: AuditInfo,
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

    fn interest_accrued_to_repayments_in_plan(
        &self,
    ) -> Result<Vec<LoanRepaymentInPlan>, LoanError> {
        let mut remaining_interest_paid = self.interest_payments();
        Ok(self
            .events
            .iter()
            .filter_map(|event| match event {
                LoanEvent::InterestIncurred {
                    amount,
                    recorded_at,
                    ..
                } => {
                    let interest_applied = std::cmp::min(*amount, remaining_interest_paid);
                    remaining_interest_paid -= interest_applied;

                    let due_at = *recorded_at;
                    let interest_outstanding_for_payment = *amount - interest_applied;
                    let status = if interest_outstanding_for_payment == UsdCents::ZERO {
                        RepaymentStatus::Paid
                    } else if Utc::now() < due_at {
                        RepaymentStatus::Due
                    } else {
                        RepaymentStatus::Overdue
                    };

                    Some(LoanRepaymentInPlan::Interest(RepaymentInPlan {
                        status,
                        outstanding: interest_outstanding_for_payment,
                        initial: *amount,
                        accrual_at: *recorded_at,
                        due_at,
                    }))
                }
                _ => None,
            })
            .collect())
    }

    fn interest_upcoming_to_repayments_in_plan(
        &self,
    ) -> Result<Vec<LoanRepaymentInPlan>, LoanError> {
        let mut interest_projections = vec![];
        let mut next_interest_period = match self.next_interest_period() {
            Ok(period) => Some(period),
            Err(LoanError::NotApprovedYet) => Err(LoanError::NotApprovedYet)?,
            Err(LoanError::AllInterestAccrualsGeneratedForLoan) | Err(_) => None,
        };
        while let Some(period) = next_interest_period {
            let interest = self.calculate_interest(period.days());
            interest_projections.push(LoanRepaymentInPlan::Interest(RepaymentInPlan {
                status: RepaymentStatus::Upcoming,
                outstanding: interest,
                initial: interest,
                accrual_at: period.end.into(),
                due_at: period.end.into(),
            }));

            next_interest_period = match self.next_period_from_end_date(period.end) {
                Ok(period) => Some(period),
                Err(LoanError::NotApprovedYet) => Err(LoanError::NotApprovedYet)?,
                Err(LoanError::AllInterestAccrualsGeneratedForLoan) | Err(_) => None,
            };
        }

        Ok(interest_projections)
    }

    fn initial_principal_to_repayment_in_plan(&self) -> Result<LoanRepaymentInPlan, LoanError> {
        let due_at = self.expires_at().ok_or(LoanError::NotApprovedYet)?;
        let status = if self.outstanding().principal == UsdCents::ZERO {
            RepaymentStatus::Paid
        } else if self.next_interest_period().is_ok() {
            RepaymentStatus::Upcoming
        } else if Utc::now() < due_at {
            RepaymentStatus::Due
        } else {
            RepaymentStatus::Overdue
        };

        Ok(LoanRepaymentInPlan::Principal(RepaymentInPlan {
            status,
            outstanding: self.outstanding().principal,
            initial: self.initial_principal(),
            accrual_at: self.approved_at().ok_or(LoanError::NotApprovedYet)?,
            due_at,
        }))
    }

    pub fn repayment_plan(&self) -> Result<Vec<LoanRepaymentInPlan>, LoanError> {
        Ok(self
            .interest_accrued_to_repayments_in_plan()?
            .into_iter()
            .chain(self.interest_upcoming_to_repayments_in_plan()?)
            .chain(std::iter::once(
                self.initial_principal_to_repayment_in_plan()?,
            ))
            .collect())
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

    fn has_user_previously_approved(&self, user_id: UserId) -> bool {
        for event in self.events.iter() {
            match event {
                LoanEvent::ApprovalAdded {
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
        price: PriceOfOneBTC,
    ) -> Result<Option<LoanApprovalData>, LoanError> {
        if self.has_user_previously_approved(approving_user_id) {
            return Err(LoanError::UserCannotApproveTwice);
        }
        if self.is_approved() {
            return Err(LoanError::AlreadyApproved);
        }

        if self.collateral() == Satoshis::ZERO {
            return Err(LoanError::NoCollateral);
        }

        let current_cvl = self.cvl(price);
        let margin_call_cvl = self.terms.margin_call_cvl;

        if current_cvl < margin_call_cvl {
            return Err(LoanError::BelowMarginLimit);
        }

        self.events.push(LoanEvent::ApprovalAdded {
            approving_user_id,
            approving_user_roles,
            audit_info,
            recorded_at: Utc::now(),
        });

        if self.approval_threshold_met() {
            let tx_ref = format!("{}-approval", self.id);
            Ok(Some(LoanApprovalData {
                initial_principal: self.initial_principal(),
                tx_ref,
                tx_id: LedgerTxId::new(),
                loan_account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
            }))
        } else {
            Ok(None)
        }
    }

    fn approval_threshold_met(&self) -> bool {
        let mut n_admin = 0;
        let mut n_bank_manager = 0;

        for event in self.events.iter() {
            if let LoanEvent::ApprovalAdded {
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

    pub fn approvals(&self) -> Vec<LoanApproval> {
        let mut loan_approvals = vec![];

        for event in self.events.iter().rev() {
            if let LoanEvent::ApprovalAdded {
                approving_user_id,
                recorded_at,
                ..
            } = event
            {
                loan_approvals.push(LoanApproval {
                    user_id: *approving_user_id,
                    approved_at: *recorded_at,
                });
            }
        }

        loan_approvals
    }

    pub(super) fn confirm_approval(
        &mut self,
        LoanApprovalData { tx_id, .. }: LoanApprovalData,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) {
        self.events.push(LoanEvent::Approved {
            tx_id,
            audit_info,
            recorded_at: executed_at,
        });
    }

    pub fn approved_at(&self) -> Option<DateTime<Utc>> {
        self.events.iter().find_map(|event| match event {
            LoanEvent::Approved { recorded_at, .. } => Some(*recorded_at),
            _ => None,
        })
    }

    pub fn next_period_from_end_date(
        &self,
        end_date: InterestPeriodEndDate,
    ) -> Result<InterestPeriod, LoanError> {
        let expiry_date = self.expires_at().ok_or(LoanError::NotApprovedYet)?;

        let calculated_start_date = end_date.next_start_date();
        let next_start_date = if calculated_start_date <= expiry_date {
            calculated_start_date
        } else {
            Err(LoanError::AllInterestAccrualsGeneratedForLoan)?
        };

        let calculated_end_date = next_start_date.absolute_end_date_for_period(self.terms.interval);
        let next_end_date = self.expires_at().map_or(calculated_end_date, |expires_at| {
            std::cmp::min(calculated_end_date, InterestPeriodEndDate::new(expires_at))
        });

        Ok(InterestPeriod::new(next_start_date, next_end_date)?)
    }

    pub fn next_interest_period(&self) -> Result<InterestPeriod, LoanError> {
        let last_end_date = InterestPeriodEndDate::new(
            self.events
                .iter()
                .rev()
                .find_map(|event| match event {
                    LoanEvent::InterestIncurred { recorded_at, .. } => Some(*recorded_at),
                    _ => self.approved_at(),
                })
                .ok_or_else(|| LoanError::NotApprovedYet)?,
        );

        self.next_period_from_end_date(last_end_date)
    }

    pub fn maybe_next_interest_period(&self) -> Result<Option<InterestPeriod>, LoanError> {
        let next_period = self.next_interest_period()?;
        Ok(next_period
            .start
            .maybe_if_before_now()
            .and(Some(next_period)))
    }

    pub fn calculate_interest(&self, days_in_interest_period: u32) -> UsdCents {
        self.terms
            .annual_rate
            .interest_for_time_period(self.initial_principal(), days_in_interest_period)
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

    pub fn initiate_interest(&self) -> Result<LoanInterestAccrual, LoanError> {
        if self.is_completed() {
            return Err(LoanError::AlreadyCompleted);
        }

        let days_in_interest_period = self
            .maybe_next_interest_period()?
            .ok_or(LoanError::InterestPeriodStartDateInFuture)?
            .days();
        let interest_for_period = self.calculate_interest(days_in_interest_period);

        let tx_ref = format!(
            "{}-interest-{}",
            self.id,
            self.count_interest_incurred() + 1
        );
        Ok(LoanInterestAccrual {
            interest: interest_for_period,
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
        audit_info: AuditInfo,
    ) {
        self.events.push(LoanEvent::InterestIncurred {
            tx_id,
            tx_ref,
            amount: interest,
            recorded_at: executed_at,
            audit_info,
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
        audit_info: AuditInfo,
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
                    audit_info,
                });
                self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, audit_info);
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
                    audit_info,
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
                    audit_info,
                    price,
                    upgrade_buffer_cvl_pct,
                );
                self.events.push(LoanEvent::Completed {
                    completed_at: recorded_at,
                    audit_info,
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
        audit_info: AuditInfo,
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
                audit_info,
            });

            return Some(*calculated_collateralization);
        }

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, audit_info)
    }

    pub fn maybe_update_collateralization(
        &mut self,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
        audit_info: AuditInfo,
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
                        audit_info,
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
                    audit_info,
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
        audit_info: AuditInfo,
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
            audit_info,
        });

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, audit_info)
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
                LoanEvent::ApprovalAdded { .. } => (),
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
    pub(super) audit_info: AuditInfo,
}

impl NewLoan {
    pub fn builder(audit_info: AuditInfo) -> NewLoanBuilder {
        let mut builder = NewLoanBuilder::default();
        builder.audit_info(audit_info);
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
                audit_info: self.audit_info,
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

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: Subject::from(UserId::new()),
        }
    }

    fn bank_manager_role() -> HashSet<Role> {
        let mut roles = HashSet::new();
        roles.insert(Role::BankManager);
        roles
    }

    fn admin_role() -> HashSet<Role> {
        let mut roles = HashSet::new();
        roles.insert(Role::Admin);
        roles
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
                    audit_info: dummy_audit_info(),
                },
                LoanEvent::InterestIncurred {
                    tx_id: LedgerTxId::new(),
                    tx_ref: "tx_ref".to_string(),
                    amount: UsdCents::from(5),
                    recorded_at: Utc::now(),
                    audit_info: dummy_audit_info(),
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
            dummy_audit_info(),
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
            dummy_audit_info(),
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
            dummy_audit_info(),
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
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let loan_approval = add_approvals(&mut loan);
        loan.confirm_approval(loan_approval, Utc::now(), dummy_audit_info());

        let third_approval = loan.add_approval(
            UserId::new(),
            bank_manager_role(),
            dummy_audit_info(),
            default_price(),
        );
        assert!(matches!(third_approval, Err(LoanError::AlreadyApproved)));
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
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let approval_time = Utc::now();

        let loan_approval = add_approvals(&mut loan);
        loan.confirm_approval(loan_approval, approval_time, dummy_audit_info());
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
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let loan_approval = add_approvals(&mut loan);
        loan.confirm_approval(loan_approval, Utc::now(), dummy_audit_info());
        assert_eq!(loan.status(), LoanStatus::Active);
        let amount = UsdCents::from(105);
        let repayment = loan.initiate_repayment(amount).unwrap();
        loan.confirm_repayment(
            repayment,
            Utc::now(),
            dummy_audit_info(),
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
            dummy_audit_info(),
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
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        assert_eq!(loan.collateral(), Satoshis::from(5000));
    }

    #[test]
    fn cannot_approve_if_loan_has_no_collateral() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let res = loan.add_approval(
            UserId::new(),
            bank_manager_role(),
            dummy_audit_info(),
            default_price(),
        );
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
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let loan_approval = add_approvals(&mut loan);
        loan.confirm_approval(loan_approval, Utc::now(), dummy_audit_info());
        loan.maybe_update_collateralization(
            default_price(),
            default_upgrade_buffer_cvl_pct(),
            dummy_audit_info(),
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
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let loan_approval = add_approvals(&mut loan);
        loan.confirm_approval(loan_approval, Utc::now(), dummy_audit_info());

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
                    dummy_audit_info()
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
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::FullyCollateralized);

            let loan_approval = add_approvals(&mut loan);
            loan.confirm_approval(loan_approval, Utc::now(), dummy_audit_info());
            assert_eq!(loan.status(), LoanStatus::Active);

            // FullyCollateralized -> None
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(7500000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_info()
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
                    dummy_audit_info()
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
                    dummy_audit_info()
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
                    dummy_audit_info()
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
                    dummy_audit_info()
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
                    dummy_audit_info()
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
                    dummy_audit_info()
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
                dummy_audit_info(),
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
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::FullyCollateralized);

            // LoanStatus::Active
            let loan_approval = add_approvals(&mut loan);
            loan.confirm_approval(loan_approval, Utc::now(), dummy_audit_info());
            assert_eq!(loan.status(), LoanStatus::Active);

            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(4350000),
                    upgrade_buffer_cvl_pct,
                    dummy_audit_info()
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
                dummy_audit_info(),
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
                    dummy_audit_info()
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
                .initiate_collateral_update(Satoshis::from(10000))
                .unwrap();
            loan.confirm_collateral_update(
                loan_collateral_update,
                Utc::now(),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::FullyCollateralized);

            let loan_approval = add_approvals(&mut loan);
            loan.confirm_approval(loan_approval, Utc::now(), dummy_audit_info());
            assert_eq!(loan.status(), LoanStatus::Active);

            // Check allowed changes from Liquidation state
            let loan_collateral_update = loan
                .initiate_collateral_update(Satoshis::from(1900))
                .unwrap();
            loan.confirm_collateral_update(
                loan_collateral_update,
                Utc::now(),
                dummy_audit_info(),
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
                    dummy_audit_info()
                ),
                None
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::UnderLiquidationThreshold);

            assert_eq!(
                loan.maybe_update_collateralization_with_liquidation_override(
                    price,
                    default_upgrade_buffer_cvl_pct(),
                    dummy_audit_info()
                ),
                Some(LoanCollaterizationState::FullyCollateralized)
            );
            let c = loan.collateralization();
            assert_eq!(c, LoanCollaterizationState::FullyCollateralized);
        }
    }

    #[test]
    fn reject_loan_approval_below_margin_limit() {
        let mut loan = Loan::try_from(init_events()).unwrap();

        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(100))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );

        let first_approval = loan.add_approval(
            UserId::new(),
            bank_manager_role(),
            dummy_audit_info(),
            default_price(),
        );
        assert!(matches!(first_approval, Err(LoanError::BelowMarginLimit)));
    }

    fn add_approvals(loan: &mut Loan) -> LoanApprovalData {
        let first_approval = loan.add_approval(
            UserId::new(),
            bank_manager_role(),
            dummy_audit_info(),
            default_price(),
        );
        assert!(first_approval.is_ok());

        let second_approval = loan.add_approval(
            UserId::new(),
            admin_role(),
            dummy_audit_info(),
            default_price(),
        );
        assert!(second_approval.is_ok());

        second_approval
            .unwrap()
            .expect("should return a loan approval")
    }

    #[test]
    fn two_admins_can_approve() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let _first_admin_approval = loan.add_approval(
            UserId::new(),
            admin_role(),
            dummy_audit_info(),
            default_price(),
        );

        let _second_admin_approval = loan.add_approval(
            UserId::new(),
            admin_role(),
            dummy_audit_info(),
            default_price(),
        );

        assert!(loan.approval_threshold_met());
    }

    #[test]
    fn admin_and_bank_manager_can_approve() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let _admin_approval = loan.add_approval(
            UserId::new(),
            admin_role(),
            dummy_audit_info(),
            default_price(),
        );

        let _bank_manager_approval = loan.add_approval(
            UserId::new(),
            bank_manager_role(),
            dummy_audit_info(),
            default_price(),
        );

        assert!(loan.approval_threshold_met());
    }

    #[test]
    fn user_with_both_admin_and_bank_manager_role_cannot_approve() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let admin_and_bank_manager = admin_role().union(&bank_manager_role()).cloned().collect();
        let _approval = loan.add_approval(
            UserId::new(),
            admin_and_bank_manager,
            dummy_audit_info(),
            default_price(),
        );

        assert!(!loan.approval_threshold_met());
    }

    #[test]
    fn two_bank_managers_cannot_approve() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let _first_bank_manager_approval = loan.add_approval(
            UserId::new(),
            bank_manager_role(),
            dummy_audit_info(),
            default_price(),
        );
        let _second_bank_manager_approval = loan.add_approval(
            UserId::new(),
            bank_manager_role(),
            dummy_audit_info(),
            default_price(),
        );

        assert!(!loan.approval_threshold_met());
    }

    #[test]
    fn same_user_cannot_approve_twice() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );

        let user_id = UserId::new();

        let first_approval = loan.add_approval(
            user_id,
            bank_manager_role(),
            dummy_audit_info(),
            default_price(),
        );

        assert!(first_approval.is_ok());

        let second_approval = loan.add_approval(
            user_id,
            bank_manager_role(),
            dummy_audit_info(),
            default_price(),
        );

        assert!(matches!(
            second_approval,
            Err(LoanError::UserCannotApproveTwice)
        ));
    }

    fn repayment_plan_events() -> EntityEvents<LoanEvent> {
        let loan_id = LoanId::new();
        EntityEvents::init(
            LoanId::new(),
            [
                LoanEvent::Initialized {
                    id: loan_id,
                    customer_id: CustomerId::new(),
                    principal: UsdCents::from(10_000_00),
                    terms: terms(),
                    account_ids: LoanAccountIds::new(),
                    customer_account_ids: CustomerLedgerAccountIds::new(),
                    audit_info: dummy_audit_info(),
                },
                LoanEvent::Approved {
                    tx_id: LedgerTxId::new(),
                    audit_info: dummy_audit_info(),
                    recorded_at: "2020-03-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap(),
                },
                LoanEvent::InterestIncurred {
                    tx_id: LedgerTxId::new(),
                    tx_ref: format!("{}-interest-{}", loan_id, 1),
                    amount: UsdCents::from(100_00),
                    recorded_at: "2020-03-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap(),
                    audit_info: dummy_audit_info(),
                },
                LoanEvent::PaymentRecorded {
                    tx_id: LedgerTxId::new(),
                    tx_ref: format!("{}-payment-{}", loan_id, 1),
                    principal_amount: UsdCents::from(4_000_00),
                    interest_amount: UsdCents::from(100_00),
                    recorded_at: "2020-04-01T14:10:00Z".parse::<DateTime<Utc>>().unwrap(),
                    audit_info: dummy_audit_info(),
                },
                LoanEvent::InterestIncurred {
                    tx_id: LedgerTxId::new(),
                    tx_ref: format!("{}-interest-{}", loan_id, 2),
                    amount: UsdCents::from(100_00),
                    recorded_at: "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap(),
                    audit_info: dummy_audit_info(),
                },
                LoanEvent::PaymentRecorded {
                    tx_id: LedgerTxId::new(),
                    tx_ref: format!("{}-payment-{}", loan_id, 1),
                    principal_amount: UsdCents::ZERO,
                    interest_amount: UsdCents::from(40_00),
                    recorded_at: "2020-04-01T14:10:00Z".parse::<DateTime<Utc>>().unwrap(),
                    audit_info: dummy_audit_info(),
                },
            ],
        )
    }

    #[test]
    fn generates_accrued_interest_as_repayments_in_plan() {
        let loan = Loan::try_from(repayment_plan_events()).unwrap();
        let interest_accrued_as_repayments = loan.interest_accrued_to_repayments_in_plan().unwrap();

        assert_eq!(interest_accrued_as_repayments.len(), 2);

        if let LoanRepaymentInPlan::Interest(first) = &interest_accrued_as_repayments[0] {
            assert_eq!(first.status, RepaymentStatus::Paid);
            assert_eq!(first.outstanding, UsdCents::from(0));
        } else {
            panic!("Expected first element to be Interest");
        }

        if let LoanRepaymentInPlan::Interest(second) = &interest_accrued_as_repayments[1] {
            assert_eq!(second.status, RepaymentStatus::Overdue);
            assert_eq!(second.outstanding, UsdCents::from(6000));
        } else {
            panic!("Expected second element to be Interest");
        }
    }

    #[test]
    fn generates_upcoming_interest_as_repayments_in_plan() {
        let loan = Loan::try_from(repayment_plan_events()).unwrap();
        let interest_upcoming_as_repayments =
            loan.interest_upcoming_to_repayments_in_plan().unwrap();

        assert_eq!(interest_upcoming_as_repayments.len(), 2);

        if let LoanRepaymentInPlan::Interest(first) = &interest_upcoming_as_repayments[0] {
            assert_eq!(first.status, RepaymentStatus::Upcoming);
            assert_eq!(first.outstanding, UsdCents::from(10164));
            assert_eq!(
                first.accrual_at,
                "2020-05-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
            )
        } else {
            panic!("Expected first element to be Interest");
        }

        if let LoanRepaymentInPlan::Interest(second) = &interest_upcoming_as_repayments[1] {
            assert_eq!(second.status, RepaymentStatus::Upcoming);
            assert_eq!(second.outstanding, UsdCents::from(4591));
            assert_eq!(
                second.accrual_at,
                "2020-06-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
            )
        } else {
            panic!("Expected second element to be Interest");
        }
    }

    #[test]
    fn generates_principal_as_repayment_in_plan() {
        let loan = Loan::try_from(repayment_plan_events()).unwrap();
        let principal_as_repayment = loan.initial_principal_to_repayment_in_plan().unwrap();

        if let LoanRepaymentInPlan::Principal(principal) = principal_as_repayment {
            assert_eq!(principal.status, RepaymentStatus::Upcoming);
            assert_eq!(principal.outstanding, UsdCents::from(6_000_00));
            assert_eq!(
                principal.accrual_at,
                "2020-03-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
            );
            assert_eq!(
                principal.due_at,
                "2020-06-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
            );
        } else {
            panic!("Expected element to be Principal");
        }
    }

    #[test]
    fn repayment_plan_for_completed_loan() {
        let mut loan = Loan::try_from(repayment_plan_events()).unwrap();

        let mut next_interest_period = loan.next_interest_period();
        while let Ok(period) = next_interest_period {
            let executed_at = period.end;

            let loan_interest_accrual = loan.initiate_interest().unwrap();
            loan.confirm_interest(
                loan_interest_accrual,
                executed_at.into(),
                dummy_audit_info(),
            );

            next_interest_period = loan.next_interest_period();
        }

        let interest_accruals = loan.interest_accrued_to_repayments_in_plan().unwrap();
        assert_eq!(interest_accruals.len(), 4);
        if let Some(LoanRepaymentInPlan::Interest(last)) = &interest_accruals.last() {
            assert_eq!(last.status, RepaymentStatus::Overdue);
            assert_eq!(last.outstanding, UsdCents::from(4591));
            assert_eq!(
                last.accrual_at,
                "2020-06-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
            )
        } else {
            panic!("Expected last element to be present and Interest");
        }

        let upcoming_interest = loan.interest_upcoming_to_repayments_in_plan().unwrap();
        assert!(upcoming_interest.is_empty());

        let principal_as_repayment = loan.initial_principal_to_repayment_in_plan().unwrap();
        if let LoanRepaymentInPlan::Principal(principal) = principal_as_repayment {
            assert_eq!(principal.status, RepaymentStatus::Overdue);
            assert_eq!(principal.outstanding, UsdCents::from(6_000_00));
            assert_eq!(
                principal.accrual_at,
                "2020-03-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
            );
            assert_eq!(
                principal.due_at,
                "2020-06-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
            );
        } else {
            panic!("Expected element to be Principal");
        }
    }
}
