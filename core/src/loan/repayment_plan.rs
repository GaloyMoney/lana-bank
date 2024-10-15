use chrono::{DateTime, Utc};

use super::{LoanEvent, UsdCents};

const INTEREST_DUE_IN: chrono::Duration = chrono::Duration::hours(24);

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoanRepaymentInPlan {
    Interest(RepaymentInPlan),
    Principal(RepaymentInPlan),
}

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a LoanEvent>,
) -> Vec<LoanRepaymentInPlan> {
    let mut terms = None;
    let mut last_interest_accrual_at = None;
    let mut approved_at = None;
    let mut outstanding_principal = UsdCents::ZERO;
    let mut initial_principal = UsdCents::ZERO;

    let mut interest_accruals = Vec::new();

    let mut interest_repayments = UsdCents::ZERO;

    for event in events {
        match event {
            LoanEvent::Initialized {
                terms: t,
                principal,
                ..
            } => {
                terms = Some(t);
                initial_principal = *principal;
                outstanding_principal += *principal;
            }
            LoanEvent::Approved { recorded_at, .. } => {
                approved_at = Some(*recorded_at);
            }
            LoanEvent::InterestIncurred {
                amount,
                recorded_at,
                ..
            } => {
                last_interest_accrual_at = Some(*recorded_at);
                let due_at = *recorded_at + INTEREST_DUE_IN;

                interest_accruals.push(RepaymentInPlan {
                    status: RepaymentStatus::Overdue,
                    outstanding: *amount,
                    initial: *amount,
                    accrual_at: *recorded_at,
                    due_at,
                });
            }
            LoanEvent::PaymentRecorded {
                interest_amount,
                principal_amount,
                ..
            } => {
                outstanding_principal -= *principal_amount;
                interest_repayments += *interest_amount;
            }
            _ => (),
        }
    }

    for payment in interest_accruals.iter_mut() {
        if interest_repayments > UsdCents::ZERO {
            let applied_payment = payment.outstanding.min(interest_repayments);
            payment.outstanding -= applied_payment;
            interest_repayments -= applied_payment;
            if payment.outstanding == UsdCents::ZERO {
                payment.status = RepaymentStatus::Paid;
            } else if Utc::now() < payment.due_at {
                payment.status = RepaymentStatus::Due;
            }
        } else {
            if Utc::now() < payment.due_at {
                payment.status = RepaymentStatus::Due;
            }
            break;
        }
    }
    let terms = terms.expect("Initialized event not found");
    let approved_at = if let Some(time) = approved_at {
        time
    } else {
        return Vec::new();
    };

    let mut res: Vec<_> = interest_accruals
        .into_iter()
        .map(LoanRepaymentInPlan::Interest)
        .collect();

    let expiry_date = terms.duration.expiration_date(approved_at);
    let last_interest_payment = last_interest_accrual_at.unwrap_or(approved_at);
    let mut next_interest_period = terms
        .accrual_interval
        .period_from(last_interest_payment)
        .next()
        .truncate(expiry_date);

    if outstanding_principal > UsdCents::ZERO {
        while let Some(period) = next_interest_period {
            let interest = terms
                .annual_rate
                .interest_for_time_period(initial_principal, period.days());

            res.push(LoanRepaymentInPlan::Interest(RepaymentInPlan {
                status: RepaymentStatus::Upcoming,
                outstanding: interest,
                initial: interest,
                accrual_at: period.end,
                due_at: period.end + INTEREST_DUE_IN,
            }));

            next_interest_period = period.next().truncate(expiry_date);
        }
    }

    res.push(LoanRepaymentInPlan::Principal(RepaymentInPlan {
        status: if outstanding_principal == UsdCents::ZERO {
            RepaymentStatus::Paid
        } else {
            RepaymentStatus::Upcoming
        },
        outstanding: outstanding_principal,
        initial: initial_principal,
        accrual_at: approved_at,
        due_at: expiry_date,
    }));

    res
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use crate::{
        ledger::{customer::*, loan::*},
        loan::*,
        primitives::*,
    };

    use super::*;

    fn terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(2))
            .accrual_interval(InterestInterval::EndOfMonth)
            .incurrence_interval(InterestInterval::EndOfMonth)
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

    fn happy_loan_events() -> Vec<LoanEvent> {
        let loan_id = LoanId::new();
        vec![
            LoanEvent::Initialized {
                id: loan_id,
                customer_id: CustomerId::new(),
                principal: UsdCents::from(1_000_000),
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
                amount: UsdCents::from(10_000),
                recorded_at: "2020-03-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
            LoanEvent::PaymentRecorded {
                tx_id: LedgerTxId::new(),
                tx_ref: format!("{}-payment-{}", loan_id, 1),
                principal_amount: UsdCents::ZERO,
                interest_amount: UsdCents::from(10_000),
                recorded_at: "2020-04-01T14:10:00Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
        ]
    }

    #[test]
    fn generates_accrued_interest_as_repayments_in_plan() {
        let events = happy_loan_events();
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 1;
        let n_future_interest_accruals = 2; //1 for april and 1 for first 14 days of may
        let n_principal_accruals = 1;
        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_future_interest_accruals + n_principal_accruals
        );
        match &repayment_plan[0] {
            LoanRepaymentInPlan::Interest(first) => {
                assert_eq!(first.status, RepaymentStatus::Paid);
                assert_eq!(first.outstanding, UsdCents::from(0));
                assert_eq!(
                    first.accrual_at,
                    "2020-03-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(
                    first.due_at,
                    "2020-04-01T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
            }
            _ => panic!("Expected first element to be Interest"),
        }
        match &repayment_plan[1] {
            LoanRepaymentInPlan::Interest(second) => {
                assert_eq!(second.status, RepaymentStatus::Upcoming);
                assert_eq!(
                    second.accrual_at,
                    "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(
                    second.due_at,
                    "2020-05-01T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
            }
            _ => panic!("Expected second element to be Interest"),
        }
        match &repayment_plan[2] {
            LoanRepaymentInPlan::Interest(third) => {
                assert_eq!(third.status, RepaymentStatus::Upcoming);
                assert_eq!(
                    third.accrual_at,
                    "2020-05-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(
                    third.due_at,
                    "2020-05-15T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
                );
            }
            _ => panic!("Expected third element to be Interest"),
        }
        match &repayment_plan[3] {
            LoanRepaymentInPlan::Principal(fourth) => {
                assert_eq!(fourth.status, RepaymentStatus::Upcoming);
                assert_eq!(
                    fourth.accrual_at,
                    "2020-03-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(
                    fourth.due_at,
                    "2020-05-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
                );
            }
            _ => panic!("Expected fourth element to be Principal"),
        }
    }

    #[test]
    fn overdue_payment() {
        let mut events = happy_loan_events();
        let loan_id = LoanId::new();
        events.push(LoanEvent::InterestIncurred {
            tx_id: LedgerTxId::new(),
            tx_ref: format!("{}-interest-{}", loan_id, 2),
            amount: UsdCents::from(10_000),
            recorded_at: "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap(),
            audit_info: dummy_audit_info(),
        });
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 1;
        let n_overdue = 1;
        let n_upcoming_interest_accruals = 1;
        let n_principal_accruals = 1;

        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals
                + n_overdue
                + n_upcoming_interest_accruals
                + n_principal_accruals
        );

        match &repayment_plan[1] {
            LoanRepaymentInPlan::Interest(second) => {
                assert_eq!(second.status, RepaymentStatus::Overdue);
                assert_eq!(second.outstanding, UsdCents::from(10_000));
                assert_eq!(
                    second.accrual_at,
                    "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(
                    second.due_at,
                    "2020-05-01T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
            }
            _ => panic!("Expected second element to be Interest"),
        }
    }

    #[test]
    fn partial_interest_payment() {
        let mut events = happy_loan_events();
        let loan_id = LoanId::new();

        let full_amount = UsdCents::from(10_000);
        let partial_amount = UsdCents::from(40_00);
        let expected_remaining_amount = full_amount - partial_amount;

        events.extend([
            LoanEvent::InterestIncurred {
                tx_id: LedgerTxId::new(),
                tx_ref: format!("{}-interest-{}", loan_id, 2),
                amount: UsdCents::from(10_000),
                recorded_at: "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
            LoanEvent::PaymentRecorded {
                tx_id: LedgerTxId::new(),
                tx_ref: format!("{}-payment-{}", loan_id, 2),
                principal_amount: UsdCents::ZERO,
                interest_amount: partial_amount,
                recorded_at: "2020-04-01T14:10:00Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 1;
        let n_overdue = 1;
        let n_upcoming_interest_accruals = 1;
        let n_principal_accruals = 1;

        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals
                + n_overdue
                + n_upcoming_interest_accruals
                + n_principal_accruals
        );

        match &repayment_plan[1] {
            LoanRepaymentInPlan::Interest(second) => {
                assert_eq!(second.status, RepaymentStatus::Overdue);
                assert_eq!(second.outstanding, expected_remaining_amount);
                assert_eq!(
                    second.accrual_at,
                    "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(
                    second.due_at,
                    "2020-05-01T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
            }
            _ => panic!("Expected second element to be Interest"),
        }
    }

    #[test]
    fn partial_principal_payment() {
        let mut events = happy_loan_events();
        let loan_id = LoanId::new();

        let full_amount = UsdCents::from(1_000_000);
        let partial_amount = UsdCents::from(100_000);
        let expected_remaining_amount = full_amount - partial_amount;

        events.extend([
            LoanEvent::InterestIncurred {
                tx_id: LedgerTxId::new(),
                tx_ref: format!("{}-interest-{}", loan_id, 2),
                amount: UsdCents::from(10_000),
                recorded_at: "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
            LoanEvent::PaymentRecorded {
                tx_id: LedgerTxId::new(),
                tx_ref: format!("{}-payment-{}", loan_id, 2),
                principal_amount: partial_amount,
                interest_amount: UsdCents::from(10_000),
                recorded_at: "2020-04-01T14:10:00Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 2;
        let n_upcoming_interest_accruals = 1;
        let n_principal_accruals = 1;

        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_upcoming_interest_accruals + n_principal_accruals
        );

        match &repayment_plan[1] {
            LoanRepaymentInPlan::Interest(second) => {
                assert_eq!(second.status, RepaymentStatus::Paid);
                assert_eq!(second.outstanding, UsdCents::ZERO);
                assert_eq!(
                    second.accrual_at,
                    "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(
                    second.due_at,
                    "2020-05-01T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
            }
            _ => panic!("Expected second element to be Interest"),
        }
        match &repayment_plan[3] {
            LoanRepaymentInPlan::Principal(fourth) => {
                assert_eq!(fourth.status, RepaymentStatus::Upcoming);
                assert_eq!(fourth.outstanding, expected_remaining_amount);
                assert_eq!(
                    fourth.accrual_at,
                    "2020-03-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(
                    fourth.due_at,
                    "2020-05-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
                );
            }
            _ => panic!("Expected fourth element to be Principal"),
        }
    }

    #[test]
    fn expect_interest_to_be_calculated_on_initial_principal() {
        let mut events = happy_loan_events();
        let loan_id = LoanId::new();
        events.extend([
            LoanEvent::InterestIncurred {
                tx_id: LedgerTxId::new(),
                tx_ref: format!("{}-interest-{}", loan_id, 2),
                amount: UsdCents::from(10_000),
                recorded_at: "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
            LoanEvent::PaymentRecorded {
                tx_id: LedgerTxId::new(),
                tx_ref: format!("{}-payment-{}", loan_id, 2),
                principal_amount: UsdCents::from(999_999),
                interest_amount: UsdCents::from(10_000),
                recorded_at: "2020-04-01T14:10:00Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 2;
        let n_upcoming_interest_accruals = 1;
        let n_principal_accruals = 1;

        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_upcoming_interest_accruals + n_principal_accruals
        );

        match &repayment_plan[2] {
            LoanRepaymentInPlan::Interest(third) => {
                assert_eq!(third.status, RepaymentStatus::Upcoming);
                assert!(third.initial > UsdCents::ONE);
            }
            _ => panic!("Expected third element to be Interest"),
        }
        match &repayment_plan[3] {
            LoanRepaymentInPlan::Principal(fourth) => {
                assert_eq!(fourth.status, RepaymentStatus::Upcoming);
                assert_eq!(fourth.outstanding, UsdCents::ONE);
            }
            _ => panic!("Expected fourth element to be Principal"),
        }
    }

    #[test]
    fn completed_loan() {
        let mut events = happy_loan_events();
        let loan_id = LoanId::new();
        events.extend([
            LoanEvent::InterestIncurred {
                tx_id: LedgerTxId::new(),
                tx_ref: format!("{}-interest-{}", loan_id, 2),
                amount: UsdCents::from(10_000),
                recorded_at: "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
            LoanEvent::InterestIncurred {
                tx_id: LedgerTxId::new(),
                tx_ref: format!("{}-interest-{}", loan_id, 3),
                amount: UsdCents::from(10_000),
                recorded_at: "2020-05-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
            LoanEvent::PaymentRecorded {
                tx_id: LedgerTxId::new(),
                tx_ref: format!("{}-payment-{}", loan_id, 2),
                principal_amount: UsdCents::from(1_000_000),
                interest_amount: UsdCents::from(20_000),
                recorded_at: "2020-05-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 3;
        let n_principal_accruals = 1;

        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_principal_accruals
        );

        match &repayment_plan[2] {
            LoanRepaymentInPlan::Interest(third) => {
                assert_eq!(third.status, RepaymentStatus::Paid);
                assert_eq!(third.outstanding, UsdCents::ZERO);
            }
            _ => panic!("Expected third element to be Interest"),
        }
        match &repayment_plan[3] {
            LoanRepaymentInPlan::Principal(fourth) => {
                assert_eq!(fourth.status, RepaymentStatus::Paid);
                assert_eq!(fourth.outstanding, UsdCents::ZERO);
            }
            _ => panic!("Expected fourth element to be Principal"),
        }
    }

    #[test]
    fn early_completed_loan() {
        let mut events = happy_loan_events();
        let loan_id = LoanId::new();
        events.extend([LoanEvent::PaymentRecorded {
            tx_id: LedgerTxId::new(),
            tx_ref: format!("{}-payment-{}", loan_id, 2),
            principal_amount: UsdCents::from(1_000_000),
            interest_amount: UsdCents::ZERO,
            recorded_at: "2020-04-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
            audit_info: dummy_audit_info(),
        }]);
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 1;
        let n_principal_accruals = 1;

        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_principal_accruals
        );

        match &repayment_plan[1] {
            LoanRepaymentInPlan::Principal(second) => {
                assert_eq!(second.status, RepaymentStatus::Paid);
                assert_eq!(second.outstanding, UsdCents::ZERO);
            }
            _ => panic!("Expected second element to be Principal"),
        }
    }
}
