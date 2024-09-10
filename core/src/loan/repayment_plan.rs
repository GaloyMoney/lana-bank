use chrono::{DateTime, Utc};

use super::{InterestPeriodStartDate, LoanEvent, UsdCents};

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

    let mut interest_payments = Vec::new();
    let mut repayments = Vec::new();

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

                interest_payments.push(RepaymentInPlan {
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
                recorded_at,
                ..
            } => {
                repayments.push((*interest_amount, *recorded_at));
                outstanding_principal -= *principal_amount;
            }
            _ => (),
        }
    }
    let mut repayment_iter = repayments.into_iter();
    for payment in interest_payments.iter_mut() {
        if let Some((amount, _)) = repayment_iter.next() {
            payment.outstanding -= amount;
            if payment.outstanding == UsdCents::ZERO {
                payment.status = RepaymentStatus::Paid;
            } else {
                if Utc::now() < payment.due_at {
                    payment.status = RepaymentStatus::Due;
                }
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
        // Early return if not approved yet
        return Vec::new();
    };

    let mut res: Vec<_> = interest_payments
        .into_iter()
        .map(LoanRepaymentInPlan::Interest)
        .collect();

    let expiry_date = terms.duration.expiration_date(approved_at);
    let last_start_date =
        InterestPeriodStartDate::new(last_interest_accrual_at.unwrap_or(approved_at));
    let mut next_interest_period = last_start_date.next_period(terms.interval, expiry_date);
    dbg!(&next_interest_period);

    while let Some(period) = next_interest_period {
        let interest = terms
            .annual_rate
            .interest_for_time_period(outstanding_principal, period.days());
        res.push(LoanRepaymentInPlan::Interest(RepaymentInPlan {
            status: RepaymentStatus::Upcoming,
            outstanding: interest,
            initial: interest,
            accrual_at: period.end.into(),
            due_at: period.end.into(),
        }));

        next_interest_period = period.end.next_period(terms.interval, expiry_date);
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

    fn happy_loan_events() -> Vec<LoanEvent> {
        let loan_id = LoanId::new();
        vec![
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
        ]
    }

    #[test]
    fn generates_accrued_interest_as_repayments_in_plan() {
        let events = happy_loan_events();
        let repayment_plan = super::project(events.iter());

        let n_existing_payments = 1;
        let n_future_interest_payments = 1;
        let n_principal_repayment = 1;
        dbg!(&repayment_plan);
        assert_eq!(
            repayment_plan.len(),
            n_existing_payments + n_future_interest_payments + n_principal_repayment
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
        // match &repayment_plan[0] {
        //     LoanRepaymentInPlan::Interest(first) => {
        //         assert_eq!(first.status, RepaymentStatus::Paid);
        //         assert_eq!(first.outstanding, UsdCents::from(0));
        //         assert_eq!(
        //             first.accrual_at,
        //             "2020-03-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
        //         );
        //         assert_eq!(
        //             first.due_at,
        //             "2020-04-01T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
        //         );
        //     }
        //     _ => panic!("Expected first element to be Interest"),
        // }
        // match &repayment_plan[2] {
        //     LoanRepaymentInPlan::Interest(first) => {
        //         assert_eq!(first.status, RepaymentStatus::Paid);
        //         assert_eq!(first.outstanding, UsdCents::from(0));
        //         assert_eq!(
        //             first.accrual_at,
        //             "2020-03-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
        //         );
        //         assert_eq!(
        //             first.due_at,
        //             "2020-04-01T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
        //         );
        //     }
        //     _ => panic!("Expected first element to be Interest"),
        // }
    }

    #[test]
    fn overdue_payment() {
        let mut events = happy_loan_events();
        // events.push( <- insert overdue interest payment
        // ASSERT OVERDUE
        // match &repayment_plan[0] {
        //     LoanRepaymentInPlan::Interest(first) => {
        //         assert_eq!(first.status, RepaymentStatus::Paid);
        //         assert_eq!(first.outstanding, UsdCents::from(0));
        //         assert_eq!(
        //             first.accrual_at,
        //             "2020-03-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
        //         );
        //         assert_eq!(
        //             first.due_at,
        //             "2020-04-01T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
        //         );
        //     }
        //     _ => panic!("Expected first element to be Interest"),
        // }
    }

    #[test]
    fn parital_repayment() {
        let mut events = happy_loan_events();
        // events.push <- interest
        // events.push <- repayment that is not enough
        // events.push( <- insert overdue interest payment
        // ASSERT OVERDUE
        // match &repayment_plan[0] {
        //     LoanRepaymentInPlan::Interest(first) => {
        //         assert_eq!(first.status, RepaymentStatus::Paid);
        //         assert_eq!(first.outstanding, UsdCents::from(0));
        //         assert_eq!(
        //             first.accrual_at,
        //             "2020-03-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
        //         );
        //         assert_eq!(
        //             first.due_at,
        //             "2020-04-01T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
        //         );
        //     }
        //     _ => panic!("Expected first element to be Interest"),
        // }
    }

    // #[test]
    // fn generates_upcoming_interest_as_repayments_in_plan() {
    //     let events = repayment_plan_events();
    //     let repayment_plan = super::project(events.iter());

    //     if let LoanRepaymentInPlan::Interest(first) = &interest_upcoming_as_repayments[0] {
    //         assert_eq!(first.status, RepaymentStatus::Upcoming);
    //         assert_eq!(first.outstanding, UsdCents::from(10164));
    //         assert_eq!(
    //             first.accrual_at,
    //             "2020-05-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
    //         )
    //     } else {
    //         panic!("Expected first element to be Interest");
    //     }

    //     if let LoanRepaymentInPlan::Interest(second) = &interest_upcoming_as_repayments[1] {
    //         assert_eq!(second.status, RepaymentStatus::Upcoming);
    //         assert_eq!(second.outstanding, UsdCents::from(4591));
    //         assert_eq!(
    //             second.accrual_at,
    //             "2020-06-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
    //         )
    //     } else {
    //         panic!("Expected second element to be Interest");
    //     }
    // }

    // #[test]
    // fn generates_principal_as_repayment_in_plan() {
    //     let loan = Loan::try_from(repayment_plan_events()).unwrap();
    //     let principal_as_repayment = loan.initial_principal_to_repayment_in_plan().unwrap();

    //     if let LoanRepaymentInPlan::Principal(principal) = principal_as_repayment {
    //         assert_eq!(principal.status, RepaymentStatus::Upcoming);
    //         assert_eq!(principal.outstanding, UsdCents::from(6_000_00));
    //         assert_eq!(
    //             principal.accrual_at,
    //             "2020-03-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
    //         );
    //         assert_eq!(
    //             principal.due_at,
    //             "2020-06-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
    //         );
    //     } else {
    //         panic!("Expected element to be Principal");
    //     }
    // }

    // #[test]
    // fn repayment_plan_for_completed_loan() {
    //     let mut loan = Loan::try_from(repayment_plan_events()).unwrap();

    //     let mut next_interest_period = loan.next_interest_period();
    //     while let Ok(Some(period)) = next_interest_period {
    //         let executed_at = period.end;

    //         let loan_interest_accrual = loan.initiate_interest().unwrap();
    //         loan.confirm_interest(
    //             loan_interest_accrual,
    //             executed_at.into(),
    //             dummy_audit_info(),
    //         );

    //         next_interest_period = loan.next_interest_period();
    //     }

    //     let interest_accruals = loan.interest_accrued_to_repayments_in_plan().unwrap();
    //     assert_eq!(interest_accruals.len(), 4);
    //     if let Some(LoanRepaymentInPlan::Interest(last)) = &interest_accruals.last() {
    //         assert_eq!(last.status, RepaymentStatus::Overdue);
    //         assert_eq!(last.outstanding, UsdCents::from(4591));
    //         assert_eq!(
    //             last.accrual_at,
    //             "2020-06-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
    //         )
    //     } else {
    //         panic!("Expected last element to be present and Interest");
    //     }

    //     let upcoming_interest = loan.interest_upcoming_to_repayments_in_plan().unwrap();
    //     assert!(upcoming_interest.is_empty());

    //     let principal_as_repayment = loan.initial_principal_to_repayment_in_plan().unwrap();
    //     if let LoanRepaymentInPlan::Principal(principal) = principal_as_repayment {
    //         assert_eq!(principal.status, RepaymentStatus::Overdue);
    //         assert_eq!(principal.outstanding, UsdCents::from(6_000_00));
    //         assert_eq!(
    //             principal.accrual_at,
    //             "2020-03-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
    //         );
    //         assert_eq!(
    //             principal.due_at,
    //             "2020-06-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
    //         );
    //     } else {
    //         panic!("Expected element to be Principal");
    //     }
    // }
}
