use chrono::{DateTime, Utc};

use crate::primitives::UsdCents;

use super::CreditFacilityEvent;

struct Accrual {
    accrued_at: DateTime<Utc>,
    remaining: UsdCents,
}

struct Payment {
    paid_at: DateTime<Utc>,
    amount: UsdCents,
}

pub struct OutstandingInterestAmounts {
    pub due: UsdCents,
    pub overdue: UsdCents,
    pub defaulted: UsdCents,
}

impl OutstandingInterestAmounts {
    pub fn total(&self) -> UsdCents {
        self.due + self.overdue + self.defaulted
    }
}

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a CreditFacilityEvent>,
) -> OutstandingInterestAmounts {
    let mut facility_terms = None;
    let mut accruals: Vec<Accrual> = Vec::new();
    let mut payments: Vec<Payment> = Vec::new();
    for event in events {
        match event {
            CreditFacilityEvent::Initialized { terms, .. } => facility_terms = Some(*terms),
            CreditFacilityEvent::InterestAccrualConcluded {
                amount, accrued_at, ..
            } => {
                accruals.push(Accrual {
                    accrued_at: *accrued_at,
                    remaining: *amount,
                });
            }
            CreditFacilityEvent::PaymentRecorded {
                interest_amount,
                recorded_at,
                ..
            } => {
                payments.push(Payment {
                    paid_at: *recorded_at,
                    amount: *interest_amount,
                });
            }
            _ => (),
        }
    }
    let terms = facility_terms.expect("Facility terms not found");

    accruals.sort_by_key(|a| a.accrued_at);
    payments.sort_by_key(|p| p.paid_at);

    for payment in payments {
        let mut remaining_payment = payment.amount;
        for accrual in accruals.iter_mut().filter(|a| a.remaining > UsdCents::ZERO) {
            if remaining_payment == UsdCents::ZERO {
                break;
            }
            let applied = std::cmp::min(accrual.remaining, remaining_payment);
            accrual.remaining -= applied;
            remaining_payment -= applied;
        }
    }

    let mut due = UsdCents::ZERO;
    let mut overdue = UsdCents::ZERO;
    let mut defaulted = UsdCents::ZERO;
    for accrual in accruals {
        if let Some(interest_overdue_duration) = terms.interest_overdue_duration {
            if interest_overdue_duration.is_past_end_date(accrual.accrued_at) {
                defaulted += accrual.remaining;
                continue;
            }
        } else if terms
            .interest_due_duration
            .is_past_end_date(accrual.accrued_at)
        {
            overdue += accrual.remaining;
        } else {
            due += accrual.remaining;
        }
    }

    OutstandingInterestAmounts {
        due,
        overdue,
        defaulted,
    }
}
