use chrono::{DateTime, Datelike, Utc};
use derive_builder::Builder;
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

pub enum LoanTransaction {
    Payment(IncrementalPayment),
    Interest(InterestAccrued),
    Collateral(CollateralUpdated),
    Origination(LoanOrigination),
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LoanCollaterization {
    NewNotCollateralized,
    NewCollateralized,
    ActiveCollateralized,
    ActiveMarginCalled,
    ActiveInLiquidation,
    Closed,
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
    Approved {
        tx_id: LedgerTxId,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    CollateralizationChanged {
        state: LoanCollaterization,
        collateral: Satoshis,
        outstanding: LoanReceivable,
        price: PriceOfOneBTC,
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
    CollateralUpdated {
        tx_id: LedgerTxId,
        tx_ref: String,
        collateral: Satoshis,
        action: CollateralAction,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    Completed {
        tx_id: LedgerTxId,
        tx_ref: String,
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
        let total = self.events.iter().fold(SignedSatoshis::ZERO, |acc, event| {
            if let LoanEvent::CollateralUpdated {
                action, collateral, ..
            } = event
            {
                let signed_collateral = SignedSatoshis::from(*collateral);
                match action {
                    CollateralAction::Add => acc + signed_collateral,
                    CollateralAction::Remove => acc - signed_collateral,
                }
            } else {
                acc
            }
        });
        Satoshis::try_from(total).expect("should be a valid satoshi amount")
    }

    pub fn transactions(&self) -> Vec<LoanTransaction> {
        let mut transactions = vec![];

        for event in self.events.iter().rev() {
            match event {
                LoanEvent::CollateralUpdated {
                    collateral,
                    action,
                    recorded_at,
                    tx_id,
                    ..
                } => match action {
                    CollateralAction::Add => {
                        transactions.push(LoanTransaction::Collateral(CollateralUpdated {
                            satoshis: *collateral,
                            action: *action,
                            recorded_at: *recorded_at,
                            tx_id: *tx_id,
                        }));
                    }
                    CollateralAction::Remove => {
                        transactions.push(LoanTransaction::Collateral(CollateralUpdated {
                            satoshis: *collateral,
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
                    transactions.push(LoanTransaction::Interest(InterestAccrued {
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
                    transactions.push(LoanTransaction::Payment(IncrementalPayment {
                        cents: *principal_amount + *interest_amount,
                        recorded_at: *transaction_recorded_at,
                        tx_id: *tx_id,
                    }));
                }

                LoanEvent::Approved {
                    tx_id, recorded_at, ..
                } => {
                    transactions.push(LoanTransaction::Origination(LoanOrigination {
                        cents: self.initial_principal(),
                        recorded_at: *recorded_at,
                        tx_id: *tx_id,
                    }));
                }

                _ => {}
            }
        }

        transactions
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

    pub fn collateralization(&self) -> (LoanCollaterization, Satoshis) {
        self.events
            .iter()
            .rev()
            .find_map(|event| match event {
                LoanEvent::CollateralizationChanged {
                    state, collateral, ..
                } => Some((*state, *collateral)),
                _ => None,
            })
            .unwrap_or((LoanCollaterization::NewNotCollateralized, self.collateral()))
    }

    pub(super) fn initiate_approval(&mut self) -> Result<LoanApproval, LoanError> {
        if self.is_approved() {
            return Err(LoanError::AlreadyApproved);
        }

        let default_price = PriceOfOneBTC::new(UsdCents::from(5000000), Utc::now());
        let default_stale_price_interval = StalePriceInterval::new(chrono::Duration::hours(1));
        let default_upgrade_buffer = CollateralUpgradeBuffer::new(5);

        self.maybe_update_collateralization(
            default_price,
            default_stale_price_interval,
            default_upgrade_buffer,
        )?;
        let (collaterization, _) = self.collateralization();
        if collaterization != LoanCollaterization::NewCollateralized {
            return Err(LoanError::NotEnoughCollateral);
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
        audit_info: AuditInfo,
    ) {
        self.events.push(LoanEvent::Approved {
            tx_id,
            audit_info,
            recorded_at: executed_at,
        });
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
        Utc::now() > self.terms.duration.expiration_date(self.created_at())
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
                self.events.push(LoanEvent::CollateralUpdated {
                    tx_id: collateral_tx_id,
                    tx_ref: collateral_tx_ref.clone(),
                    collateral,
                    action: CollateralAction::Remove,
                    recorded_at,
                    audit_info,
                });
                self.events.push(LoanEvent::CollateralizationChanged {
                    state: LoanCollaterization::Closed,
                    collateral,
                    outstanding: self.outstanding(),
                    price: PriceOfOneBTC::ZERO,
                });
                self.events.push(LoanEvent::Completed {
                    tx_id: collateral_tx_id,
                    tx_ref: collateral_tx_ref,
                    completed_at: recorded_at,
                    audit_info,
                });
            }
        }
    }

    pub fn cvl(
        &mut self,
        price: PriceOfOneBTC,
        stale_price_interval: StalePriceInterval,
    ) -> Result<CVLPct, LoanError> {
        let (is_stale, now) = price.is_stale(stale_price_interval);
        if is_stale {
            return Err(LoanError::StalePrice(price, now));
        }

        let collateral_value =
            price.try_sats_to_cents(self.collateral(), rust_decimal::RoundingStrategy::ToZero)?;

        if collateral_value == UsdCents::ZERO {
            Ok(CVLPct::ZERO)
        } else {
            Ok(CVLPct::from_loan_amounts(
                collateral_value,
                self.outstanding().total(),
            ))
        }
    }

    pub fn maybe_update_collateralization(
        &mut self,
        price: PriceOfOneBTC,
        stale_price_interval: StalePriceInterval,
        upgrade_buffer: CollateralUpgradeBuffer,
    ) -> Result<Option<LoanCollaterization>, LoanError> {
        let loan_status = self.status();

        let margin_call_cvl = self.terms.margin_call_cvl;
        let liquidation_cvl = self.terms.liquidation_cvl;
        let current_cvl = self.cvl(price, stale_price_interval)?;

        let (current_collateralization, collateral) = self.collateralization();
        let calculated_collateralization = match (current_cvl, loan_status) {
            _ if current_cvl == CVLPct::ZERO => {
                if loan_status == LoanStatus::New {
                    &LoanCollaterization::NewNotCollateralized
                } else {
                    &LoanCollaterization::Closed
                }
            }
            _ if current_cvl >= margin_call_cvl => {
                if loan_status == LoanStatus::New {
                    &LoanCollaterization::NewCollateralized
                } else {
                    &LoanCollaterization::ActiveCollateralized
                }
            }
            _ if current_cvl >= liquidation_cvl => {
                if loan_status == LoanStatus::New {
                    &LoanCollaterization::NewNotCollateralized
                } else {
                    &LoanCollaterization::ActiveMarginCalled
                }
            }
            _ => {
                if loan_status == LoanStatus::New {
                    &LoanCollaterization::NewNotCollateralized
                } else {
                    &LoanCollaterization::ActiveInLiquidation
                }
            }
        };

        match (current_collateralization, *calculated_collateralization) {
            // Invalid "New loan" state changes
            (
                LoanCollaterization::NewNotCollateralized,
                LoanCollaterization::ActiveMarginCalled,
            )
            | (
                LoanCollaterization::NewNotCollateralized,
                LoanCollaterization::ActiveInLiquidation,
            )
            | (LoanCollaterization::NewCollateralized, LoanCollaterization::ActiveMarginCalled)
            | (LoanCollaterization::NewCollateralized, LoanCollaterization::ActiveInLiquidation) => {
                Ok(None)
            }

            // Valid "New loan" state changes
            (LoanCollaterization::NewNotCollateralized, LoanCollaterization::NewCollateralized)
            | (
                LoanCollaterization::NewNotCollateralized,
                LoanCollaterization::ActiveCollateralized,
            )
            | (LoanCollaterization::NewNotCollateralized, LoanCollaterization::Closed)
            | (LoanCollaterization::NewCollateralized, LoanCollaterization::NewNotCollateralized)
            | (LoanCollaterization::NewCollateralized, LoanCollaterization::ActiveCollateralized)
            | (LoanCollaterization::NewCollateralized, LoanCollaterization::Closed) => {
                self.events.push(LoanEvent::CollateralizationChanged {
                    state: *calculated_collateralization,
                    collateral,
                    outstanding: self.outstanding(),
                    price,
                });

                Ok(Some(*calculated_collateralization))
            }

            // Valid active collateral degraded changes
            (
                LoanCollaterization::ActiveCollateralized,
                LoanCollaterization::ActiveMarginCalled,
            )
            | (
                LoanCollaterization::ActiveCollateralized,
                LoanCollaterization::ActiveInLiquidation,
            )
            | (LoanCollaterization::ActiveMarginCalled, LoanCollaterization::ActiveInLiquidation) =>
            {
                self.events.push(LoanEvent::CollateralizationChanged {
                    state: *calculated_collateralization,
                    collateral,
                    outstanding: self.outstanding(),
                    price,
                });

                Ok(Some(*calculated_collateralization))
            }

            // Valid active collateral upgraded changes
            (
                LoanCollaterization::ActiveMarginCalled,
                LoanCollaterization::ActiveCollateralized,
            ) => {
                if margin_call_cvl.is_significantly_lower_than(current_cvl, upgrade_buffer) {
                    self.events.push(LoanEvent::CollateralizationChanged {
                        state: *calculated_collateralization,
                        collateral,
                        outstanding: self.outstanding(),
                        price,
                    });

                    Ok(Some(*calculated_collateralization))
                } else {
                    Ok(None)
                }
            }

            // Invalid active collateral changes back to new
            (
                LoanCollaterization::ActiveCollateralized,
                LoanCollaterization::NewNotCollateralized,
            )
            | (LoanCollaterization::ActiveCollateralized, LoanCollaterization::NewCollateralized)
            | (
                LoanCollaterization::ActiveMarginCalled,
                LoanCollaterization::NewNotCollateralized,
            )
            | (LoanCollaterization::ActiveMarginCalled, LoanCollaterization::NewCollateralized)
            | (
                LoanCollaterization::ActiveInLiquidation,
                LoanCollaterization::NewNotCollateralized,
            )
            | (LoanCollaterization::ActiveInLiquidation, LoanCollaterization::NewCollateralized) => {
                Ok(None)
            }

            // No collateral change
            (
                LoanCollaterization::NewNotCollateralized,
                LoanCollaterization::NewNotCollateralized,
            )
            | (LoanCollaterization::NewCollateralized, LoanCollaterization::NewCollateralized)
            | (
                LoanCollaterization::ActiveCollateralized,
                LoanCollaterization::ActiveCollateralized,
            )
            | (LoanCollaterization::ActiveMarginCalled, LoanCollaterization::ActiveMarginCalled)
            | (
                LoanCollaterization::ActiveInLiquidation,
                LoanCollaterization::ActiveInLiquidation,
            )
            | (LoanCollaterization::Closed, LoanCollaterization::Closed) => Ok(None),

            // Valid collateral changes to closed
            (_, LoanCollaterization::Closed) => {
                self.events.push(LoanEvent::CollateralizationChanged {
                    state: *calculated_collateralization,
                    collateral,
                    outstanding: self.outstanding(),
                    price,
                });

                Ok(Some(*calculated_collateralization))
            }

            // Invalid state changes
            (_, LoanCollaterization::NewNotCollateralized)
            | (LoanCollaterization::ActiveInLiquidation, _)
            | (LoanCollaterization::Closed, _) => Ok(None),
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
            collateral,
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
            collateral,
            action,
            ..
        }: LoanCollateralUpdate,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
        price: Option<PriceOfOneBTC>, // TODO: add price service and remove Option
        stale_price_interval: Option<StalePriceInterval>, // TODO: pass from config and remove Option
        upgrade_buffer: Option<CollateralUpgradeBuffer>, // TODO: pass from config and remove Option
    ) -> Result<Option<LoanCollaterization>, LoanError> {
        self.events.push(LoanEvent::CollateralUpdated {
            tx_id,
            tx_ref,
            collateral,
            action,
            recorded_at: executed_at,
            audit_info,
        });

        let default_price = match price {
            Some(price) => price,
            None => PriceOfOneBTC::new(UsdCents::from(5000000), Utc::now()),
        };
        let default_stale_price_interval = match stale_price_interval {
            Some(stale_price_interval) => stale_price_interval,
            None => StalePriceInterval::new(chrono::Duration::hours(1)),
        };
        let default_upgrade_buffer = match upgrade_buffer {
            Some(upgrade_buffer) => upgrade_buffer,
            None => CollateralUpgradeBuffer::new(5),
        };

        self.maybe_update_collateralization(
            default_price,
            default_stale_price_interval,
            default_upgrade_buffer,
        )
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

    fn default_price() -> Option<PriceOfOneBTC> {
        Some(PriceOfOneBTC::new(UsdCents::from(5000000), Utc::now()))
    }

    fn default_stale_price_interval() -> Option<StalePriceInterval> {
        Some(StalePriceInterval::new(chrono::Duration::hours(1)))
    }

    fn default_upgrade_buffer() -> Option<CollateralUpgradeBuffer> {
        Some(CollateralUpgradeBuffer::new(5))
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
        loan.confirm_repayment(repayment, Utc::now(), dummy_audit_info());
        assert_eq!(
            loan.outstanding(),
            LoanReceivable {
                principal: UsdCents::from(100),
                interest: UsdCents::from(1)
            }
        );
        let amount = UsdCents::from(2);
        let repayment = loan.initiate_repayment(amount).unwrap();
        loan.confirm_repayment(repayment, Utc::now(), dummy_audit_info());
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
        loan.confirm_repayment(repayment, Utc::now(), dummy_audit_info());
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
            default_stale_price_interval(),
            default_upgrade_buffer(),
        )
        .unwrap();
        let loan_approval = loan.initiate_approval();
        assert!(loan_approval.is_ok());
        loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_info());

        let loan_approval = loan.initiate_approval();
        assert!(loan_approval.is_err())
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
            default_stale_price_interval(),
            default_upgrade_buffer(),
        )
        .unwrap();
        let loan_approval = loan.initiate_approval().unwrap();
        loan.confirm_approval(loan_approval, Utc::now(), dummy_audit_info());
        assert_eq!(loan.status(), LoanStatus::Active);
        let amount = UsdCents::from(105);
        let repayment = loan.initiate_repayment(amount).unwrap();
        loan.confirm_repayment(repayment, Utc::now(), dummy_audit_info());
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
            default_stale_price_interval(),
            default_upgrade_buffer(),
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
            default_stale_price_interval(),
            default_upgrade_buffer(),
        );
        assert_eq!(loan.collateral(), Satoshis::from(5000));
    }

    #[test]
    fn cannot_approve_if_loan_has_no_collateral() {
        let loan = Loan::try_from(init_events()).unwrap();
        let res = loan.initiate_approval();
        assert!(matches!(res, Err(LoanError::NotEnoughCollateral)));
    }

    #[test]
    fn test_collateralization() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        assert_eq!(
            loan.collateralization(),
            (LoanCollaterization::NewNotCollateralized, Satoshis::ZERO)
        );

        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(12000000))
            .unwrap();
        let loan_approval = loan.initiate_approval();
        loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_info());
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_stale_price_interval(),
            default_upgrade_buffer(),
        )
        .unwrap();
        let loan_approval = loan.initiate_approval();
        loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_info());
        assert_eq!(
            loan.collateralization(),
            (
                LoanCollaterization::ActiveCollateralized,
                Satoshis::from(12000000)
            )
        );
    }

    #[test]
    fn calculate_cvl() {
        let stale_price_interval = &StalePriceInterval::new(chrono::Duration::hours(1));

        let mut loan = Loan::try_from(init_events()).unwrap();
        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(3000))
            .unwrap();
        loan.confirm_collateral_update(
            loan_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_stale_price_interval(),
            default_upgrade_buffer(),
        );
        let loan_approval = loan.initiate_approval();
        loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_info());

        let expected_cvl = CVLPct::from(dec!(142));
        let cvl = loan
            .cvl(
                PriceOfOneBTC::new(UsdCents::from(5000000), Utc::now()),
                *stale_price_interval,
            )
            .unwrap();
        assert_eq!(cvl, expected_cvl);

        let expected_cvl = CVLPct::from(dec!(100));
        let cvl = loan
            .cvl(
                PriceOfOneBTC::new(UsdCents::from(3500000), Utc::now()),
                *stale_price_interval,
            )
            .unwrap();
        assert_eq!(cvl, expected_cvl);
    }

    mod maybe_update_collateralization {

        use crate::primitives::StalePriceInterval;

        use super::*;

        fn price_from(value: u64) -> PriceOfOneBTC {
            PriceOfOneBTC::new(UsdCents::from(value), Utc::now())
        }

        #[test]
        fn test_transitions_on_active_loan() {
            let stale_price_interval = &StalePriceInterval::new(chrono::Duration::hours(1));
            let upgrade_buffer = CollateralUpgradeBuffer::new(5);

            let mut loan = Loan::try_from(init_events()).unwrap();
            assert_eq!(
                loan.collateralization(),
                (LoanCollaterization::NewNotCollateralized, Satoshis::ZERO)
            );
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(7500000),
                    *stale_price_interval,
                    upgrade_buffer,
                )
                .unwrap(),
                None
            );
            let loan_collateral_update = loan
                .initiate_collateral_update(Satoshis::from(3000))
                .unwrap();
            loan.confirm_collateral_update(
                loan_collateral_update,
                Utc::now(),
                dummy_audit_info(),
                default_price(),
                default_stale_price_interval(),
                default_upgrade_buffer(),
            )
            .unwrap();
            let loan_approval = loan.initiate_approval();
            loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_info());
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::ActiveCollateralized);

            // Collateralized -> None
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(7500000),
                    *stale_price_interval,
                    upgrade_buffer,
                )
                .unwrap(),
                None
            );
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::ActiveCollateralized);

            // Collateralized -> MarginCalled
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(4350000),
                    *stale_price_interval,
                    upgrade_buffer,
                )
                .unwrap(),
                Some(LoanCollaterization::ActiveMarginCalled)
            );
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::ActiveMarginCalled);

            // MarginCalled -> None (CVL above margin_call but below marginal increase)
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(4500000),
                    *stale_price_interval,
                    upgrade_buffer,
                )
                .unwrap(),
                None
            );
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::ActiveMarginCalled);

            // MarginCalled -> Collateralized (CVL above margin_call and marginal increase)
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(7000000),
                    *stale_price_interval,
                    upgrade_buffer,
                )
                .unwrap(),
                Some(LoanCollaterization::ActiveCollateralized)
            );
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::ActiveCollateralized);

            // MarginCalled -> InLiquidation
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(3000000),
                    *stale_price_interval,
                    upgrade_buffer,
                )
                .unwrap(),
                Some(LoanCollaterization::ActiveInLiquidation)
            );
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::ActiveInLiquidation);

            // InLiquidation -> None (CVL above Collaterization requirement)
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(10000000),
                    *stale_price_interval,
                    upgrade_buffer,
                )
                .unwrap(),
                None
            );
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::ActiveInLiquidation);
        }

        #[test]
        fn test_transitions_on_non_active_loans() {
            let stale_price_interval = &StalePriceInterval::new(chrono::Duration::hours(1));
            let upgrade_buffer = CollateralUpgradeBuffer::new(5);

            // LoanStatus::New
            let mut loan = Loan::try_from(init_events()).unwrap();

            assert_eq!(loan.status(), LoanStatus::New);
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(6500000),
                    *stale_price_interval,
                    upgrade_buffer,
                )
                .unwrap(),
                None
            );
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::NewNotCollateralized);

            // LoanStatus::Active
            let loan_collateral_update = loan
                .initiate_collateral_update(Satoshis::from(3000))
                .unwrap();
            loan.confirm_collateral_update(
                loan_collateral_update,
                Utc::now(),
                dummy_audit_info(),
                default_price(),
                default_stale_price_interval(),
                default_upgrade_buffer(),
            )
            .unwrap();
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::NewCollateralized);

            let loan_approval = loan.initiate_approval();
            loan.confirm_approval(loan_approval.unwrap(), Utc::now(), dummy_audit_info());
            assert_eq!(loan.status(), LoanStatus::Active);
            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(4350000),
                    *stale_price_interval,
                    upgrade_buffer,
                )
                .unwrap(),
                Some(LoanCollaterization::ActiveMarginCalled)
            );
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::ActiveMarginCalled);

            // LoanStatus::Closed
            assert_eq!(loan.status(), LoanStatus::Active);
            let repayment = loan.initiate_repayment(loan.outstanding().total()).unwrap();
            loan.confirm_repayment(repayment, Utc::now(), dummy_audit_info());
            assert_eq!(loan.status(), LoanStatus::Closed);
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::Closed);

            assert_eq!(
                loan.maybe_update_collateralization(
                    price_from(10000000),
                    *stale_price_interval,
                    upgrade_buffer,
                )
                .unwrap(),
                None
            );
            let (c, _) = loan.collateralization();
            assert_eq!(c, LoanCollaterization::Closed);
        }
    }
}
