use chrono::{DateTime, Datelike, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::*,
    ledger::{customer::CustomerLedgerAccountIds, loan::LoanAccountIds},
    primitives::*,
};

use super::error::LoanError;
use super::terms::TermValues;

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
        start_date: DateTime<Utc>,
    },
    TermsUpdated {
        terms: TermValues,
    },
    Approved {
        tx_id: LedgerTxId,
        initial_collateral: Satoshis,
    },
    InterestIncurred {
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
    },
    PaymentRecorded {
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
    },
    Completed {
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
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
    pub start_date: DateTime<Utc>,
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

    fn payments(&self) -> UsdCents {
        self.events
            .iter()
            .filter_map(|event| match event {
                LoanEvent::PaymentRecorded { amount, .. } => Some(*amount),
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

    pub fn outstanding(&self) -> UsdCents {
        self.initial_principal() + self.interest_recorded() - self.payments()
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

    pub(super) fn approve(
        &mut self,
        tx_id: LedgerTxId,
        initial_collateral: Satoshis,
    ) -> Result<(), LoanError> {
        if self.is_approved() {
            return Err(LoanError::AlreadyApproved);
        }
        self.events.push(LoanEvent::Approved {
            tx_id,
            initial_collateral,
        });

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
        Utc::now() > self.terms.duration.expiration_date(self.start_date)
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
                .next_interest_payment(self.start_date)
                .day()
                - self.start_date.day()
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
        tx_id: LedgerTxId,
        record_amount: UsdCents,
    ) -> Result<String, LoanError> {
        for event in self.events.iter() {
            if let LoanEvent::Completed { .. } = event {
                return Err(LoanError::AlreadyCompleted);
            }
        }

        let outstanding = self.outstanding();

        if outstanding < record_amount {
            return Err(LoanError::PaymentExceedsOutstandingLoanAmount(
                record_amount,
                outstanding,
            ));
        }

        let tx_ref = format!("{}-payment-{}", self.id, self.count_recorded_payments() + 1);
        self.events.push(LoanEvent::PaymentRecorded {
            tx_id,
            tx_ref: tx_ref.clone(),
            amount: record_amount,
        });
        if outstanding == record_amount {
            self.events.push(LoanEvent::Completed {
                tx_id,
                tx_ref: tx_ref.clone(),
                amount: record_amount,
            });
        }
        Ok(tx_ref)
    }

    fn count_recorded_payments(&self) -> usize {
        self.events
            .iter()
            .filter(|event| matches!(event, LoanEvent::PaymentRecorded { .. }))
            .count()
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
                    start_date,
                    terms,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .customer_id(*customer_id)
                        .terms(terms.clone())
                        .account_ids(*account_ids)
                        .customer_account_ids(*customer_account_ids)
                        .start_date(*start_date);
                }
                LoanEvent::TermsUpdated { terms } => {
                    builder = builder.terms(terms.clone());
                }
                LoanEvent::Approved { .. } => (),
                LoanEvent::InterestIncurred { .. } => (),
                LoanEvent::PaymentRecorded { .. } => (),
                LoanEvent::Completed { .. } => (),
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
    #[builder(default = "Utc::now()")]
    start_date: DateTime<Utc>,
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
                start_date: self.start_date,
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
            [LoanEvent::Initialized {
                id: LoanId::new(),
                customer_id: CustomerId::new(),
                principal: UsdCents::try_from_usd(dec!(100)).unwrap(),
                terms: terms(),
                account_ids: LoanAccountIds::new(),
                customer_account_ids: CustomerLedgerAccountIds::new(),
                start_date: Utc::now(),
            }],
        )
    }

    #[test]
    fn outstanding() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        assert_eq!(
            loan.outstanding(),
            UsdCents::try_from_usd(dec!(100)).unwrap()
        );
        let _ = loan.record_if_not_exceeding_outstanding(
            LedgerTxId::new(),
            UsdCents::try_from_usd(dec!(50)).unwrap(),
        );
        assert_eq!(
            loan.outstanding(),
            UsdCents::try_from_usd(dec!(50)).unwrap()
        );

        let _ = loan.record_if_not_exceeding_outstanding(
            LedgerTxId::new(),
            UsdCents::try_from_usd(dec!(50)).unwrap(),
        );
        assert_eq!(loan.outstanding(), UsdCents::ZERO);
        assert!(loan.is_completed());
    }

    #[test]
    fn prevent_double_approve() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        let res = loan.approve(
            LedgerTxId::new(),
            Satoshis::try_from_btc(dec!(0.12)).unwrap(),
        );
        assert!(res.is_ok());

        let res = loan.approve(
            LedgerTxId::new(),
            Satoshis::try_from_btc(dec!(0.12)).unwrap(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_status() {
        let mut loan = Loan::try_from(init_events()).unwrap();
        assert_eq!(loan.status(), LoanStatus::New);
        let _ = loan.approve(
            LedgerTxId::new(),
            Satoshis::try_from_btc(dec!(0.12)).unwrap(),
        );
        assert_eq!(loan.status(), LoanStatus::Active);
        let _ = loan.record_if_not_exceeding_outstanding(
            LedgerTxId::new(),
            UsdCents::try_from_usd(dec!(100)).unwrap(),
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
