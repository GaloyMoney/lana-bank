use chrono::{DateTime, Utc};

use super::{
    InterestPeriodStartDate, LoanEvent, LoanReceivable, LoanRepaymentInPlan, RepaymentInPlan,
    RepaymentStatus, TermValues, UsdCents,
};

fn initial_terms<'a>(
    events: &(impl DoubleEndedIterator<Item = &'a LoanEvent> + Clone),
) -> TermValues {
    if let Some(LoanEvent::Initialized { terms, .. }) = events.clone().next() {
        *terms
    } else {
        unreachable!("Initialized event not found")
    }
}

pub fn initial_principal<'a>(
    events: &(impl DoubleEndedIterator<Item = &'a LoanEvent> + Clone),
) -> UsdCents {
    if let Some(LoanEvent::Initialized { principal, .. }) = events.clone().next() {
        *principal
    } else {
        unreachable!("Initialized event not found")
    }
}

fn principal_payments<'a>(
    events: &(impl DoubleEndedIterator<Item = &'a LoanEvent> + Clone),
) -> UsdCents {
    events
        .clone()
        .filter_map(|event| match event {
            LoanEvent::PaymentRecorded {
                principal_amount, ..
            } => Some(*principal_amount),
            _ => None,
        })
        .fold(UsdCents::ZERO, |acc, amount| acc + amount)
}

fn interest_payments<'a>(
    events: &(impl DoubleEndedIterator<Item = &'a LoanEvent> + Clone),
) -> UsdCents {
    events
        .clone()
        .filter_map(|event| match event {
            LoanEvent::PaymentRecorded {
                interest_amount, ..
            } => Some(*interest_amount),
            _ => None,
        })
        .fold(UsdCents::ZERO, |acc, amount| acc + amount)
}

fn interest_recorded<'a>(
    events: &(impl DoubleEndedIterator<Item = &'a LoanEvent> + Clone),
) -> UsdCents {
    events
        .clone()
        .filter_map(|event| match event {
            LoanEvent::InterestIncurred { amount, .. } => Some(*amount),
            _ => None,
        })
        .fold(UsdCents::ZERO, |acc, amount| acc + amount)
}

fn interest_accrued_to_repayments_in_plan<'a>(
    events: &(impl DoubleEndedIterator<Item = &'a LoanEvent> + Clone),
    expiry_date: DateTime<Utc>,
) -> Vec<LoanRepaymentInPlan> {
    let interval = initial_terms(events).interval;

    let mut remaining_interest_paid = interest_payments(events);
    events
        .clone()
        .filter_map(|event| match event {
            LoanEvent::InterestIncurred {
                amount,
                recorded_at,
                ..
            } => {
                let interest_applied = std::cmp::min(*amount, remaining_interest_paid);
                remaining_interest_paid -= interest_applied;

                let interest_outstanding_for_payment = *amount - interest_applied;
                let due_at = match InterestPeriodStartDate::new(*recorded_at)
                    .current_period(interval, expiry_date)
                {
                    Some(period) => period.end,
                    None => return None,
                };

                let status = if interest_outstanding_for_payment == UsdCents::ZERO {
                    RepaymentStatus::Paid
                } else if due_at > Utc::now() {
                    RepaymentStatus::Due
                } else {
                    RepaymentStatus::Overdue
                };

                Some(LoanRepaymentInPlan::Interest(RepaymentInPlan {
                    status,
                    outstanding: interest_outstanding_for_payment,
                    initial: *amount,
                    accrual_at: *recorded_at,
                    due_at: due_at.into(),
                }))
            }
            _ => None,
        })
        .collect()
}

fn interest_upcoming_to_repayments_in_plan<'a>(
    events: &(impl DoubleEndedIterator<Item = &'a LoanEvent> + Clone),
    approval_date: DateTime<Utc>,
    expiry_date: DateTime<Utc>,
) -> Vec<LoanRepaymentInPlan> {
    let principal = initial_principal(events);
    let TermValues {
        interval,
        annual_rate,
        ..
    } = initial_terms(events);

    let last_start_date = InterestPeriodStartDate::new(
        events
            .clone()
            .rev()
            .find_map(|event| match event {
                LoanEvent::InterestIncurred { recorded_at, .. } => Some(*recorded_at),
                _ => None,
            })
            .unwrap_or(approval_date),
    );
    let mut next_interest_period = last_start_date.next_period(interval, expiry_date);

    let mut interest_projections = vec![];
    while let Some(period) = next_interest_period {
        let interest = annual_rate.interest_for_time_period(principal, period.days());
        interest_projections.push(LoanRepaymentInPlan::Interest(RepaymentInPlan {
            status: RepaymentStatus::Upcoming,
            outstanding: interest,
            initial: interest,
            accrual_at: period.end.into(),
            due_at: period.end.into(),
        }));

        next_interest_period = period.end.next_period(interval, expiry_date);
    }

    interest_projections
}

fn initial_principal_to_repayment_in_plan<'a>(
    events: &(impl DoubleEndedIterator<Item = &'a LoanEvent> + Clone),
    approval_date: DateTime<Utc>,
    expiry_date: DateTime<Utc>,
) -> LoanRepaymentInPlan {
    let interval = initial_terms(events).interval;
    let principal = initial_principal(events);

    let outstanding = LoanReceivable {
        principal: principal - principal_payments(events),
        interest: interest_recorded(events) - interest_payments(events),
    };

    let last_start_date = InterestPeriodStartDate::new(
        events
            .clone()
            .rev()
            .find_map(|event| match event {
                LoanEvent::InterestIncurred { recorded_at, .. } => Some(*recorded_at),
                _ => None,
            })
            .unwrap_or(approval_date),
    );
    let next_interest_period = last_start_date.next_period(interval, expiry_date);

    let status = if outstanding.principal == UsdCents::ZERO {
        RepaymentStatus::Paid
    } else if next_interest_period.is_some() {
        RepaymentStatus::Upcoming
    } else if Utc::now() < expiry_date {
        RepaymentStatus::Due
    } else {
        RepaymentStatus::Overdue
    };

    LoanRepaymentInPlan::Principal(RepaymentInPlan {
        status,
        outstanding: outstanding.principal,
        initial: principal,
        accrual_at: approval_date,
        due_at: expiry_date,
    })
}

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a LoanEvent> + Clone,
) -> Vec<LoanRepaymentInPlan> {
    let approval_date = match events.clone().find_map(|event| match event {
        LoanEvent::Approved { recorded_at, .. } => Some(*recorded_at),
        _ => None,
    }) {
        Some(date) => date,
        None => return Default::default(),
    };
    let expiry_date = initial_terms(&events)
        .duration
        .expiration_date(approval_date);

    interest_accrued_to_repayments_in_plan(&events, expiry_date)
        .into_iter()
        .chain(interest_upcoming_to_repayments_in_plan(
            &events,
            approval_date,
            expiry_date,
        ))
        .chain(std::iter::once(initial_principal_to_repayment_in_plan(
            &events,
            approval_date,
            expiry_date,
        )))
        .collect()
}
