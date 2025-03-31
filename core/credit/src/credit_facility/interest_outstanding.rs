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
            CreditFacilityEvent::Initialized { terms, .. } => facility_terms = Some(**terms),
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

#[cfg(test)]
mod tests {
    use audit::{AuditEntryId, AuditInfo};
    use rust_decimal_macros::dec;

    use crate::{primitives::*, terms::*, CreditFacilityAccountIds};

    use super::*;

    fn terms(
        interest_due_duration: InterestDuration,
        interest_overdue_duration: Option<InterestDuration>,
    ) -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(2))
            .interest_due_duration(interest_due_duration)
            .interest_overdue_duration(interest_overdue_duration)
            .accrual_interval(InterestInterval::EndOfMonth)
            .incurrence_interval(InterestInterval::EndOfDay)
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .one_time_fee_rate(OneTimeFeeRatePct::new(5))
            .build()
            .expect("should build a valid term")
    }

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn initial_credit_facility_events(
        activated_at: DateTime<Utc>,
        interest_due_duration: InterestDuration,
        interest_overdue_duration: Option<InterestDuration>,
    ) -> Vec<CreditFacilityEvent> {
        let credit_facility_id = CreditFacilityId::new();
        let first_disbursal_idx = DisbursalIdx::FIRST;
        vec![
            CreditFacilityEvent::Initialized {
                id: credit_facility_id,
                customer_id: CustomerId::new(),
                account_ids: CreditFacilityAccountIds::new(),
                facility: UsdCents::from(1_000_000),
                terms: Box::new(terms(interest_due_duration, interest_overdue_duration)),
                audit_info: dummy_audit_info(),
                disbursal_credit_account_id: CalaAccountId::new(),
                approval_process_id: ApprovalProcessId::new(),
            },
            CreditFacilityEvent::Activated {
                ledger_tx_id: LedgerTxId::new(),
                activated_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalInitiated {
                approval_process_id: ApprovalProcessId::new(),
                disbursal_id: DisbursalId::new(),
                idx: first_disbursal_idx,
                amount: UsdCents::from(1000),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalConcluded {
                idx: first_disbursal_idx,
                tx_id: Some(LedgerTxId::new()),
                recorded_at: Utc::now(),
                audit_info: dummy_audit_info(),
                canceled: false,
            },
        ]
    }

    #[test]
    fn no_payment_interest_due() {
        let activated_at = crate::time::now();
        let interest_due_duration = InterestDuration::Days(30);
        let mut events = initial_credit_facility_events(activated_at, interest_due_duration, None);

        let first_interest_idx = InterestAccrualIdx::FIRST;
        let first_interest_accrued_at = activated_at;
        events.push(CreditFacilityEvent::InterestAccrualConcluded {
            idx: first_interest_idx,
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::from(2),
            accrued_at: first_interest_accrued_at,
            audit_info: dummy_audit_info(),
        });
        let outstanding = project(events.iter());

        assert_eq!(outstanding.due, UsdCents::from(2));
        assert_eq!(outstanding.overdue, UsdCents::ZERO);
        assert_eq!(outstanding.defaulted, UsdCents::ZERO);
        assert_eq!(outstanding.total(), UsdCents::from(2));
    }

    #[test]
    fn no_payment_interest_overdue() {
        let activated_at = crate::time::now();
        let interest_due_duration = InterestDuration::Days(0);
        let mut events = initial_credit_facility_events(activated_at, interest_due_duration, None);

        let first_interest_idx = InterestAccrualIdx::FIRST;
        let first_interest_accrued_at = activated_at;
        events.push(CreditFacilityEvent::InterestAccrualConcluded {
            idx: first_interest_idx,
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::from(2),
            accrued_at: first_interest_accrued_at,
            audit_info: dummy_audit_info(),
        });
        let outstanding = project(events.iter());

        assert_eq!(outstanding.due, UsdCents::ZERO);
        assert_eq!(outstanding.overdue, UsdCents::from(2));
        assert_eq!(outstanding.defaulted, UsdCents::ZERO);
        assert_eq!(outstanding.total(), UsdCents::from(2));
    }

    #[test]
    fn no_payment_interest_defaulted() {
        let activated_at = crate::time::now();
        let interest_due_duration = InterestDuration::Days(0);
        let interest_overdue_duration = Some(InterestDuration::Days(0));
        let mut events = initial_credit_facility_events(
            activated_at,
            interest_due_duration,
            interest_overdue_duration,
        );

        let first_interest_idx = InterestAccrualIdx::FIRST;
        let first_interest_accrued_at = activated_at;
        events.push(CreditFacilityEvent::InterestAccrualConcluded {
            idx: first_interest_idx,
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::from(2),
            accrued_at: first_interest_accrued_at,
            audit_info: dummy_audit_info(),
        });
        let outstanding = project(events.iter());

        assert_eq!(outstanding.due, UsdCents::ZERO);
        assert_eq!(outstanding.overdue, UsdCents::ZERO);
        assert_eq!(outstanding.defaulted, UsdCents::from(2));
        assert_eq!(outstanding.total(), UsdCents::from(2));
    }

    #[test]
    fn full_payment_clears_interest() {
        let activated_at = crate::time::now();
        let interest_due_duration = InterestDuration::Days(30);
        let mut events = initial_credit_facility_events(activated_at, interest_due_duration, None);

        let first_interest_idx = InterestAccrualIdx::FIRST;
        let first_interest_accrued_at = activated_at;
        events.push(CreditFacilityEvent::InterestAccrualConcluded {
            idx: first_interest_idx,
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::from(2),
            accrued_at: first_interest_accrued_at,
            audit_info: dummy_audit_info(),
        });
        let outstanding = project(events.iter());
        assert_eq!(outstanding.total(), UsdCents::from(2));

        events.push(CreditFacilityEvent::PaymentRecorded {
            payment_id: PaymentId::new(),
            disbursal_amount: UsdCents::ZERO,
            interest_amount: UsdCents::from(2),
            audit_info: dummy_audit_info(),
            recorded_at: first_interest_accrued_at,
        });
        let outstanding = project(events.iter());
        assert_eq!(outstanding.total(), UsdCents::ZERO);
    }

    #[test]
    fn no_payment_interests_due_and_overdue() {
        let activated_at = crate::time::now() - chrono::Duration::days(1);
        let interest_due_duration = InterestDuration::Days(1);
        let mut events = initial_credit_facility_events(activated_at, interest_due_duration, None);

        let first_interest_idx = InterestAccrualIdx::FIRST;
        let first_interest_accrued_at = activated_at;
        let second_interest_idx = first_interest_idx.next();
        let second_interest_accrued_at = activated_at + chrono::Duration::days(1);
        events.extend([
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: first_interest_idx,
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(2),
                accrued_at: first_interest_accrued_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: second_interest_idx,
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(4),
                accrued_at: second_interest_accrued_at,
                audit_info: dummy_audit_info(),
            },
        ]);
        let outstanding = project(events.iter());

        assert_eq!(outstanding.due, UsdCents::from(4));
        assert_eq!(outstanding.overdue, UsdCents::from(2));
        assert_eq!(outstanding.defaulted, UsdCents::ZERO);
        assert_eq!(outstanding.total(), UsdCents::from(6));
    }

    #[test]
    fn partial_payment_clears_overdue_first() {
        let activated_at = crate::time::now() - chrono::Duration::days(1);
        let interest_due_duration = InterestDuration::Days(1);
        let mut events = initial_credit_facility_events(activated_at, interest_due_duration, None);

        let first_interest_idx = InterestAccrualIdx::FIRST;
        let first_interest_accrued_at = activated_at;
        let second_interest_idx = first_interest_idx.next();
        let second_interest_accrued_at = activated_at + chrono::Duration::days(1);
        events.extend([
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: first_interest_idx,
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(2),
                accrued_at: first_interest_accrued_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: second_interest_idx,
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(4),
                accrued_at: second_interest_accrued_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::PaymentRecorded {
                payment_id: PaymentId::new(),
                disbursal_amount: UsdCents::ZERO,
                interest_amount: UsdCents::from(3),
                audit_info: dummy_audit_info(),
                recorded_at: second_interest_accrued_at,
            },
        ]);
        let outstanding = project(events.iter());

        assert_eq!(outstanding.due, UsdCents::from(3));
        assert_eq!(outstanding.overdue, UsdCents::ZERO);
        assert_eq!(outstanding.defaulted, UsdCents::ZERO);
        assert_eq!(outstanding.total(), UsdCents::from(3));
    }
}
