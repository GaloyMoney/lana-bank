use chrono::{DateTime, Utc};

use crate::CreditFacilityReceivable;

use super::{CreditFacilityEvent, UsdCents};

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
pub enum CreditFacilityRepaymentInPlan {
    Disbursal(RepaymentInPlan),
    Interest(RepaymentInPlan),
}

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a CreditFacilityEvent>,
) -> Vec<CreditFacilityRepaymentInPlan> {
    let mut terms = None;
    let mut activated_at = None;

    let mut disbursed_amounts = std::collections::HashMap::new();
    let mut total_disbursed = UsdCents::ZERO;
    let mut due_and_outstanding_disbursed = UsdCents::ZERO;

    let mut last_interest_accrual_at = None;
    let mut interest_accruals = Vec::new();
    let mut due_and_outstanding_interest = UsdCents::ZERO;

    for event in events {
        match event {
            CreditFacilityEvent::Initialized { terms: t, .. } => {
                terms = Some(t);
            }
            CreditFacilityEvent::Activated {
                activated_at: recorded_at,
                ..
            } => {
                activated_at = Some(*recorded_at);
            }
            CreditFacilityEvent::DisbursalInitiated { idx, amount, .. } => {
                disbursed_amounts.insert(*idx, *amount);
            }
            CreditFacilityEvent::DisbursalConcluded { idx, .. } => {
                if let Some(amount) = disbursed_amounts.remove(idx) {
                    total_disbursed += amount;
                    due_and_outstanding_disbursed += amount;
                }
            }
            CreditFacilityEvent::InterestAccrualConcluded {
                amount, accrued_at, ..
            } => {
                last_interest_accrual_at = Some(*accrued_at);
                let due_at = *accrued_at;

                interest_accruals.push(RepaymentInPlan {
                    status: RepaymentStatus::Overdue,
                    initial: *amount,
                    outstanding: *amount,
                    accrual_at: *accrued_at,
                    due_at,
                });
            }
            CreditFacilityEvent::PaymentRecorded {
                interest_amount,
                disbursal_amount,
                ..
            } => {
                due_and_outstanding_disbursed -= *disbursal_amount;
                due_and_outstanding_interest += *interest_amount;
            }
            _ => {}
        }
    }

    for payment in interest_accruals.iter_mut() {
        if due_and_outstanding_interest > UsdCents::ZERO {
            let applied_payment = payment.outstanding.min(due_and_outstanding_interest);
            payment.outstanding -= applied_payment;
            due_and_outstanding_interest -= applied_payment;
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

    let due_and_outstanding = CreditFacilityReceivable {
        disbursed: due_and_outstanding_disbursed,
        interest: due_and_outstanding_interest,
    };
    let terms = terms.expect("Initialized event not found");
    let activated_at = match activated_at {
        Some(time) => time,
        None => return Vec::new(),
    };

    let mut res: Vec<_> = interest_accruals
        .into_iter()
        .map(CreditFacilityRepaymentInPlan::Interest)
        .collect();

    let expiry_date = terms.duration.expiration_date(activated_at);
    let last_interest_payment = last_interest_accrual_at.unwrap_or(activated_at);
    let mut next_interest_period = terms
        .accrual_interval
        .period_from(last_interest_payment)
        .next()
        .truncate(expiry_date);

    if !due_and_outstanding.is_zero() {
        while let Some(period) = next_interest_period {
            let interest = terms
                .annual_rate
                .interest_for_time_period(due_and_outstanding.total(), period.days());

            res.push(CreditFacilityRepaymentInPlan::Interest(RepaymentInPlan {
                status: RepaymentStatus::Upcoming,
                initial: interest,
                outstanding: interest,
                accrual_at: period.end,
                due_at: period.end,
            }));

            next_interest_period = period.next().truncate(expiry_date);
        }
    }

    res.push(CreditFacilityRepaymentInPlan::Disbursal(RepaymentInPlan {
        status: if due_and_outstanding_disbursed == UsdCents::ZERO {
            RepaymentStatus::Paid
        } else {
            RepaymentStatus::Upcoming
        },
        initial: total_disbursed,
        outstanding: due_and_outstanding_disbursed,
        accrual_at: expiry_date,
        due_at: expiry_date,
    }));

    res
}

#[cfg(test)]
mod tests {
    use audit::{AuditEntryId, AuditInfo};
    use chrono::{Datelike, TimeZone, Utc};
    use rust_decimal_macros::dec;

    use crate::{primitives::*, terms::*, CreditFacilityAccountIds};

    use super::*;

    fn terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(2))
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
            sub: Subject::from(UserId::new()).to_string(),
        }
    }

    fn default_activated_at() -> DateTime<Utc> {
        "2020-03-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
    }

    fn end_of_month(start_date: DateTime<Utc>) -> DateTime<Utc> {
        let current_year = start_date.year();
        let current_month = start_date.month();

        let (year, month) = if current_month == 12 {
            (current_year + 1, 1)
        } else {
            (current_year, current_month + 1)
        };

        Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0)
            .single()
            .expect("should return a valid date time")
            - chrono::Duration::seconds(1)
    }

    fn happy_credit_facility_events() -> Vec<CreditFacilityEvent> {
        let credit_facility_id = CreditFacilityId::new();
        let activated_at = default_activated_at();
        let deposit_account_id = DepositAccountId::new();
        let first_disbursal_idx = DisbursalIdx::FIRST;
        let first_interest_idx = InterestAccrualIdx::FIRST;
        let first_interest_accrued_at = end_of_month(activated_at);
        vec![
            CreditFacilityEvent::Initialized {
                id: credit_facility_id,
                customer_id: CustomerId::new(),
                account_ids: CreditFacilityAccountIds::new(),
                facility: UsdCents::from(1_000_000),
                terms: terms(),
                audit_info: dummy_audit_info(),
                deposit_account_id,
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
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: first_interest_idx,
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(2),
                accrued_at: first_interest_accrued_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::PaymentRecorded {
                payment_id: PaymentId::new(),
                disbursal_amount: UsdCents::ZERO,
                interest_amount: UsdCents::from(2),
                audit_info: dummy_audit_info(),
                recorded_at: first_interest_accrued_at,
            },
        ]
    }

    #[test]
    fn generates_repayments_in_plan() {
        let events = happy_credit_facility_events();
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 1;
        let n_future_interest_accruals = 2;
        let n_principal_accruals = 1;
        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_future_interest_accruals + n_principal_accruals
        );
        match &repayment_plan[0] {
            CreditFacilityRepaymentInPlan::Interest(first) => {
                assert_eq!(first.status, RepaymentStatus::Paid);
                assert_eq!(first.initial, UsdCents::from(2));
                assert_eq!(first.outstanding, UsdCents::from(0));
                assert_eq!(
                    first.accrual_at,
                    "2020-03-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(first.due_at, first.accrual_at,);
            }
            _ => panic!("Expected first element to be Interest"),
        }
        match &repayment_plan[1] {
            CreditFacilityRepaymentInPlan::Interest(second) => {
                assert_eq!(second.status, RepaymentStatus::Upcoming);
                assert_eq!(second.initial, UsdCents::from(10));
                assert_eq!(second.outstanding, UsdCents::from(10));
                assert_eq!(
                    second.accrual_at,
                    "2020-04-30T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(second.due_at, second.accrual_at);
            }
            _ => panic!("Expected second element to be Interest"),
        }
        match &repayment_plan[2] {
            CreditFacilityRepaymentInPlan::Interest(third) => {
                assert_eq!(third.status, RepaymentStatus::Upcoming);
                assert_eq!(third.initial, UsdCents::from(5));
                assert_eq!(third.outstanding, UsdCents::from(5));
                assert_eq!(
                    third.accrual_at,
                    "2020-05-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(third.due_at, third.accrual_at);
            }
            _ => panic!("Expected third element to be Interest"),
        }
        match &repayment_plan[3] {
            CreditFacilityRepaymentInPlan::Disbursal(fourth) => {
                assert_eq!(fourth.status, RepaymentStatus::Upcoming);
                assert_eq!(fourth.initial, UsdCents::from(1000));
                assert_eq!(fourth.outstanding, UsdCents::from(1000));
                assert_eq!(
                    fourth.accrual_at,
                    "2020-05-14T14:20:00Z".parse::<DateTime<Utc>>().unwrap()
                );
                assert_eq!(fourth.due_at, fourth.accrual_at);
            }
            _ => panic!("Expected fourth element to be Disbursal"),
        }
    }

    #[test]
    fn overdue_payment() {
        let mut events = happy_credit_facility_events();
        let first_interest_accrued_at = end_of_month(default_activated_at());
        let second_interest_idx = InterestAccrualIdx::FIRST.next();
        let second_interest_accrued_at =
            end_of_month(first_interest_accrued_at + chrono::Duration::days(1));
        events.extend([CreditFacilityEvent::InterestAccrualConcluded {
            idx: second_interest_idx,
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::from(12),
            accrued_at: second_interest_accrued_at,
            audit_info: dummy_audit_info(),
        }]);
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 2;
        let n_future_interest_accruals = 1;
        let n_principal_accruals = 1;
        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_future_interest_accruals + n_principal_accruals
        );
        match &repayment_plan[1] {
            CreditFacilityRepaymentInPlan::Interest(second) => {
                assert_eq!(second.status, RepaymentStatus::Overdue);
            }
            _ => panic!("Expected second element to be Interest"),
        }
    }

    #[test]
    fn partial_interest_payment() {
        let mut events = happy_credit_facility_events();
        let first_interest_accrued_at = end_of_month(default_activated_at());
        let second_interest_idx = InterestAccrualIdx::FIRST.next();
        let second_interest_accrued_at =
            end_of_month(first_interest_accrued_at + chrono::Duration::days(1));
        events.extend([
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: second_interest_idx,
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(12),
                accrued_at: second_interest_accrued_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::PaymentRecorded {
                payment_id: PaymentId::new(),
                disbursal_amount: UsdCents::ZERO,
                interest_amount: UsdCents::from(2),
                audit_info: dummy_audit_info(),
                recorded_at: second_interest_accrued_at,
            },
        ]);
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 2;
        let n_future_interest_accruals = 1;
        let n_principal_accruals = 1;
        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_future_interest_accruals + n_principal_accruals
        );
        match &repayment_plan[1] {
            CreditFacilityRepaymentInPlan::Interest(second) => {
                assert_eq!(second.status, RepaymentStatus::Overdue);
                assert_eq!(second.initial, UsdCents::from(12));
                assert_eq!(second.outstanding, UsdCents::from(10));
            }
            _ => panic!("Expected second element to be Interest"),
        }
    }

    #[test]
    fn increase_disbursal() {
        let mut events = happy_credit_facility_events();
        let second_disbursal_idx = DisbursalIdx::FIRST.next();
        let second_disbursal_at = default_activated_at() + chrono::Duration::days(1);
        events.extend([
            CreditFacilityEvent::DisbursalInitiated {
                disbursal_id: DisbursalId::new(),
                approval_process_id: ApprovalProcessId::new(),
                idx: second_disbursal_idx,
                amount: UsdCents::from(2000),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalConcluded {
                idx: second_disbursal_idx,
                tx_id: Some(LedgerTxId::new()),
                recorded_at: second_disbursal_at,
                audit_info: dummy_audit_info(),
                canceled: false,
            },
        ]);
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 1;
        let n_future_interest_accruals = 2;
        let n_principal_accruals = 1;
        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_future_interest_accruals + n_principal_accruals
        );
        match &repayment_plan[3] {
            CreditFacilityRepaymentInPlan::Disbursal(fourth) => {
                assert_eq!(fourth.initial, UsdCents::from(3000));
                assert_eq!(fourth.outstanding, UsdCents::from(3000));
            }
            _ => panic!("Expected fourth element to be Disbursal"),
        }
    }

    #[test]
    fn partial_principal_payment() {
        let mut events = happy_credit_facility_events();
        let first_interest_accrued_at = end_of_month(default_activated_at());
        let second_interest_idx = InterestAccrualIdx::FIRST.next();
        let second_interest_accrued_at =
            end_of_month(first_interest_accrued_at + chrono::Duration::days(1));
        let third_interest_accrued_at =
            end_of_month(second_interest_accrued_at + chrono::Duration::days(1));
        events.extend([
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: second_interest_idx,
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(12),
                accrued_at: second_interest_accrued_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::PaymentRecorded {
                payment_id: PaymentId::new(),
                disbursal_amount: UsdCents::ZERO,
                interest_amount: UsdCents::from(12),
                audit_info: dummy_audit_info(),
                recorded_at: second_interest_accrued_at,
            },
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: second_interest_idx.next(),
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(6),
                accrued_at: third_interest_accrued_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::PaymentRecorded {
                payment_id: PaymentId::new(),
                disbursal_amount: UsdCents::from(100),
                interest_amount: UsdCents::from(6),
                audit_info: dummy_audit_info(),
                recorded_at: third_interest_accrued_at,
            },
        ]);
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 3;
        let n_future_interest_accruals = 0;
        let n_principal_accruals = 1;
        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_future_interest_accruals + n_principal_accruals
        );

        match &repayment_plan[3] {
            CreditFacilityRepaymentInPlan::Disbursal(fourth) => {
                assert_eq!(fourth.initial, UsdCents::from(1000));
                assert_eq!(fourth.outstanding, UsdCents::from(900));
            }
            _ => panic!("Expected fourth element to be Disbursal"),
        }
    }

    #[test]
    fn completed_facility() {
        let mut events = happy_credit_facility_events();
        let first_interest_accrued_at = end_of_month(default_activated_at());
        let second_interest_idx = InterestAccrualIdx::FIRST.next();
        let second_interest_accrued_at =
            end_of_month(first_interest_accrued_at + chrono::Duration::days(1));
        let third_interest_accrued_at =
            end_of_month(second_interest_accrued_at + chrono::Duration::days(1));
        events.extend([
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: second_interest_idx,
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(12),
                accrued_at: second_interest_accrued_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::PaymentRecorded {
                payment_id: PaymentId::new(),
                disbursal_amount: UsdCents::ZERO,
                interest_amount: UsdCents::from(12),
                audit_info: dummy_audit_info(),
                recorded_at: second_interest_accrued_at,
            },
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: second_interest_idx.next(),
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(6),
                accrued_at: third_interest_accrued_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::PaymentRecorded {
                payment_id: PaymentId::new(),
                disbursal_amount: UsdCents::from(1000),
                interest_amount: UsdCents::from(6),
                audit_info: dummy_audit_info(),
                recorded_at: third_interest_accrued_at,
            },
        ]);
        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 3;
        let n_future_interest_accruals = 0;
        let n_principal_accruals = 1;
        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_future_interest_accruals + n_principal_accruals
        );

        match &repayment_plan[3] {
            CreditFacilityRepaymentInPlan::Disbursal(fourth) => {
                assert_eq!(fourth.status, RepaymentStatus::Paid);
            }
            _ => panic!("Expected fourth element to be Disbursal"),
        }
    }
}
