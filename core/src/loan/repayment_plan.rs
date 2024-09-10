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
            // Currently assuming every repayment covers the interest for exactly 1 interest
            // accrual. Needs to updated with unit tests.
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

    let mut interest_projections = vec![];
    while let Some(period) = next_interest_period {
        let interest = terms
            .annual_rate
            .interest_for_time_period(outstanding_principal, period.days());
        interest_projections.push(LoanRepaymentInPlan::Interest(RepaymentInPlan {
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
