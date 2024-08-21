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

use super::{error::LoanError, terms::TermValues};

#[derive(Debug, Clone, PartialEq)]
pub struct LoanReceivable {
    pub principal: UsdCents,
    pub interest: UsdCents,
}

pub struct LoanTransaction {
    pub amount: TransactionAmount,
    pub transaction_type: TransactionType,
    pub recorded_at: DateTime<Utc>,
}

pub enum TransactionAmount {
    Sats(Satoshis),
    Cents(UsdCents),
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum TransactionType {
    InterestPayment,
    PrincipalPayment,
    CollateralAdded,
    CollateralRemoved,
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
    },
    Approved {
        tx_id: LedgerTxId,
    },
    InterestIncurred {
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
    },
    PaymentRecorded {
        tx_id: LedgerTxId,
        tx_ref: String,
        principal_amount: UsdCents,
        interest_amount: UsdCents,
        transaction_recorded_at: DateTime<Utc>,
    },
    CollateralUpdated {
        tx_id: LedgerTxId,
        tx_ref: String,
        collateral: Satoshis,
        action: CollateralAction,
        recorded_at: DateTime<Utc>,
    },
    Completed {
        tx_id: LedgerTxId,
        tx_ref: String,
        completed_at: DateTime<Utc>,
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
        let mut transacions = vec![];

        for event in self.events.iter().rev() {
            match event {
                LoanEvent::PaymentRecorded {
                    principal_amount,
                    interest_amount,
                    transaction_recorded_at,
                    ..
                } => {
                    if *principal_amount != UsdCents::ZERO {
                        transacions.push(LoanTransaction {
                            amount: TransactionAmount::Cents(*principal_amount),
                            transaction_type: TransactionType::PrincipalPayment,
                            recorded_at: *transaction_recorded_at,
                        });
                    }
                    if *interest_amount != UsdCents::ZERO {
                        transacions.push(LoanTransaction {
                            amount: TransactionAmount::Cents(*interest_amount),
                            transaction_type: TransactionType::InterestPayment,
                            recorded_at: *transaction_recorded_at,
                        });
                    }
                }

                LoanEvent::CollateralUpdated {
                    collateral,
                    action,
                    recorded_at,
                    ..
                } => match action {
                    CollateralAction::Add => {
                        transacions.push(LoanTransaction {
                            amount: TransactionAmount::Sats(*collateral),
                            transaction_type: TransactionType::CollateralAdded,
                            recorded_at: *recorded_at,
                        });
                    }
                    CollateralAction::Remove => {
                        transacions.push(LoanTransaction {
                            amount: TransactionAmount::Sats(*collateral),
                            transaction_type: TransactionType::CollateralRemoved,
                            recorded_at: *recorded_at,
                        });
                    }
                },

                _ => {}
            }
        }

        transacions
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

    pub(super) fn approve(&mut self, tx_id: LedgerTxId) -> Result<(), LoanError> {
        if self.is_approved() {
            return Err(LoanError::AlreadyApproved);
        }

        if self.collateral() == Satoshis::ZERO {
            return Err(LoanError::NoCollateral);
        }

        self.events.push(LoanEvent::Approved { tx_id });

        Ok(())
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

    pub fn add_interest(&mut self, tx_id: LedgerTxId) -> Result<(UsdCents, String), LoanError> {
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
        self.events.push(LoanEvent::InterestIncurred {
            tx_id,
            tx_ref: tx_ref.clone(),
            amount: interest,
        });
        Ok((interest, tx_ref))
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
        transaction_recorded_at: DateTime<Utc>,
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
                    transaction_recorded_at,
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
                    transaction_recorded_at,
                });
                self.events.push(LoanEvent::CollateralUpdated {
                    tx_id: collateral_tx_id,
                    tx_ref: collateral_tx_ref.clone(),
                    collateral,
                    action: CollateralAction::Remove,
                    recorded_at: transaction_recorded_at,
                });
                self.events.push(LoanEvent::Completed {
                    tx_id: collateral_tx_id,
                    tx_ref: collateral_tx_ref,
                    completed_at: transaction_recorded_at,
                });
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
    ) {
        self.events.push(LoanEvent::CollateralUpdated {
            tx_id,
            tx_ref,
            collateral,
            action,
            recorded_at: executed_at,
        });
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
}

impl NewLoan {
    pub fn builder() -> NewLoanBuilder {
        NewLoanBuilder::default()
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
                },
                LoanEvent::InterestIncurred {
                    tx_id: LedgerTxId::new(),
                    tx_ref: "tx_ref".to_string(),
                    amount: UsdCents::from(5),
                },
            ],
        )
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
        loan.confirm_repayment(repayment, Utc::now());
        assert_eq!(
            loan.outstanding(),
            LoanReceivable {
                principal: UsdCents::from(100),
                interest: UsdCents::from(1)
            }
        );
        let amount = UsdCents::from(2);
        let repayment = loan.initiate_repayment(amount).unwrap();
        loan.confirm_repayment(repayment, Utc::now());
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
        loan.confirm_repayment(repayment, Utc::now());
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
        loan.confirm_collateral_update(loan_collateral_update, Utc::now());
        let res = loan.approve(LedgerTxId::new());
        assert!(res.is_ok());

        let res = loan.approve(LedgerTxId::new());
        assert!(res.is_err());
    }

    #[test]
    fn status() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        assert_eq!(loan.status(), LoanStatus::New);
        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        loan.confirm_collateral_update(loan_collateral_update, Utc::now());
        let _ = loan.approve(LedgerTxId::new());
        assert_eq!(loan.status(), LoanStatus::Active);
        let amount = UsdCents::from(105);
        let repayment = loan.initiate_repayment(amount).unwrap();
        loan.confirm_repayment(repayment, Utc::now());
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
        loan.confirm_collateral_update(loan_collateral_update, Utc::now());
        assert_eq!(loan.collateral(), Satoshis::from(10000));

        let loan_collateral_update = loan
            .initiate_collateral_update(Satoshis::from(5000))
            .unwrap();
        loan.confirm_collateral_update(loan_collateral_update, Utc::now());
        assert_eq!(loan.collateral(), Satoshis::from(5000));
    }

    #[test]
    fn cannot_approve_if_loan_has_no_collateral() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let res = loan.approve(LedgerTxId::new());
        assert!(matches!(res, Err(LoanError::NoCollateral)));
    }
}
