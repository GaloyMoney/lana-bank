use chrono::{DateTime, Datelike, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::*,
    ledger::{
        customer::CustomerLedgerAccountIds,
        loan::{LoanAccountIds, LoanPayment},
    },
    primitives::*,
};

use super::error::LoanError;
use super::terms::TermValues;

#[derive(Debug, Clone, PartialEq)]
pub struct LoanReceivable {
    pub principal: UsdCents,
    pub interest: UsdCents,
}

impl LoanReceivable {
    pub fn total(&self) -> UsdCents {
        self.interest + self.principal
    }

    fn allocate_payment(&self, amount: UsdCents) -> Result<LoanPayment, LoanError> {
        let mut remaining = amount;

        let interest = std::cmp::min(amount, self.interest);
        remaining -= interest;

        let principal = std::cmp::min(remaining, self.principal);
        remaining -= principal;

        Ok(LoanPayment {
            interest,
            principal,
        })
    }
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum CollateralAction {
    Add,
    Remove,
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
    TermsUpdated {
        terms: TermValues,
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
    },
    Completed {
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
    },

    CollateralAdjusted {
        tx_id: LedgerTxId,
        tx_ref: String,
        action: CollateralAction,
        collateral: Satoshis,
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

    pub fn collateral(&self) -> Result<Satoshis, ConversionError> {
        let mut current = SignedSatoshis::ZERO;

        for event in self.events.iter() {
            if let LoanEvent::CollateralAdjusted {
                action, collateral, ..
            } = event
            {
                let signed_collateral = SignedSatoshis::from(*collateral);
                match action {
                    CollateralAction::Add => current = current + signed_collateral,
                    CollateralAction::Remove => current = current - signed_collateral,
                }
            }
        }

        Satoshis::try_from(current)
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

    #[allow(dead_code)]
    pub(super) fn set_terms(&mut self, terms: TermValues) {
        self.terms = terms.clone();
        self.events.push(LoanEvent::TermsUpdated { terms });
    }

    pub(super) fn approve(&mut self, tx_id: LedgerTxId) -> Result<(), LoanError> {
        if self.is_approved() {
            return Err(LoanError::AlreadyApproved);
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

    pub fn record_if_not_exceeding_outstanding(
        &mut self,
        payment_tx_id: LedgerTxId,
        collateral_tx_id: LedgerTxId,
        record_amount: UsdCents,
    ) -> Result<(String, String, LoanPayment), LoanError> {
        for event in self.events.iter() {
            if let LoanEvent::Completed { .. } = event {
                return Err(LoanError::AlreadyCompleted);
            }
        }

        let outstanding = self.outstanding();

        if outstanding.total() < record_amount {
            return Err(LoanError::PaymentExceedsOutstandingLoanAmount(
                record_amount,
                outstanding.total(),
            ));
        }

        let payment = outstanding.allocate_payment(record_amount)?;

        let payment_tx_ref: &str =
            &format!("{}-payment-{}", self.id, self.count_recorded_payments() + 1);
        self.events.push(LoanEvent::PaymentRecorded {
            tx_id: payment_tx_id,
            tx_ref: payment_tx_ref.to_string(),
            principal_amount: payment.principal,
            interest_amount: payment.interest,
        });

        let collateral_tx_ref: &str = &format!("{}-collateral-returned", self.id);
        if outstanding.total() == record_amount {
            self.events.push(LoanEvent::Completed {
                tx_id: collateral_tx_id,
                tx_ref: collateral_tx_ref.to_string(),
                amount: record_amount,
            });
        }

        Ok((
            payment_tx_ref.to_string(),
            collateral_tx_ref.to_string(),
            payment,
        ))
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
            .filter(|event| matches!(event, LoanEvent::CollateralAdjusted { .. }))
            .count()
    }

    pub(super) fn adjust_collateral(
        &mut self,
        tx_id: LedgerTxId,
        collateral: Satoshis,
        action: CollateralAction,
    ) -> Result<String, LoanError> {
        let tx_ref = format!(
            "{}-collateral-{}",
            self.id,
            self.count_collateral_adjustments() + 1
        );
        self.events.push(LoanEvent::CollateralAdjusted {
            tx_id,
            tx_ref: tx_ref.clone(),
            action,
            collateral,
        });
        Ok(tx_ref)
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
                LoanEvent::TermsUpdated { terms } => {
                    builder = builder.terms(terms.clone());
                }
                LoanEvent::Approved { .. } => (),
                LoanEvent::InterestIncurred { .. } => (),
                LoanEvent::PaymentRecorded { .. } => (),
                LoanEvent::Completed { .. } => (),
                LoanEvent::CollateralAdjusted { .. } => (),
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

    use crate::loan::{AnnualRatePct, Duration, InterestInterval};

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
        let _ = loan.record_if_not_exceeding_outstanding(
            LedgerTxId::new(),
            LedgerTxId::new(),
            UsdCents::from(4),
        );
        assert_eq!(
            loan.outstanding(),
            LoanReceivable {
                principal: UsdCents::from(100),
                interest: UsdCents::from(1)
            }
        );

        let _ = loan.record_if_not_exceeding_outstanding(
            LedgerTxId::new(),
            LedgerTxId::new(),
            UsdCents::from(2),
        );
        assert_eq!(
            loan.outstanding(),
            LoanReceivable {
                principal: UsdCents::from(99),
                interest: UsdCents::ZERO
            }
        );

        let loan_payment = loan.record_if_not_exceeding_outstanding(
            LedgerTxId::new(),
            LedgerTxId::new(),
            UsdCents::from(100),
        );
        assert!(loan_payment.is_err());

        let _ = loan.record_if_not_exceeding_outstanding(
            LedgerTxId::new(),
            LedgerTxId::new(),
            UsdCents::from(99),
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
        let res = loan.approve(LedgerTxId::new());
        assert!(res.is_ok());

        let res = loan.approve(LedgerTxId::new());
        assert!(res.is_err());
    }

    #[test]
    fn test_status() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        assert_eq!(loan.status(), LoanStatus::New);
        let _ = loan.approve(LedgerTxId::new());
        assert_eq!(loan.status(), LoanStatus::Active);
        let _ = loan.record_if_not_exceeding_outstanding(
            LedgerTxId::new(),
            LedgerTxId::new(),
            UsdCents::from(105),
        );
        assert_eq!(loan.status(), LoanStatus::Closed);
    }

    #[test]
    fn can_update_terms() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        assert_eq!(loan.terms.annual_rate, AnnualRatePct::from(dec!(12)));

        let new_terms = TermValues::builder()
            .annual_rate(dec!(15))
            .duration(Duration::Months(3))
            .interval(InterestInterval::EndOfMonth)
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term");
        loan.set_terms(new_terms);

        assert_eq!(
            loan.events
                .iter()
                .filter_map(|event| match event {
                    LoanEvent::TermsUpdated { .. } => Some(()),
                    _ => None,
                })
                .count(),
            1
        );

        assert!(matches!(
            loan.events.iter().last().unwrap(),
            LoanEvent::TermsUpdated { .. }
        ));

        assert_eq!(loan.terms.annual_rate, AnnualRatePct::from(dec!(15)));
    }
}
