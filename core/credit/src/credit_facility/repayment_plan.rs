use chrono::{DateTime, Utc};

use crate::primitives::*;

use super::{CreditFacilityEvent, CreditFacilityReceivable};

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
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(_),
                balance_type: BalanceUpdatedType::Disbursal,
                amount,
                ..
            } => {
                total_disbursed += *amount;
                due_and_outstanding_disbursed += *amount;
            }
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(_),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount,
                updated_at: posted_at,
                ..
            } => {
                last_interest_accrual_at = Some(*posted_at);
                let due_at = *posted_at;

                interest_accruals.push(RepaymentInPlan {
                    status: RepaymentStatus::Overdue,
                    initial: *amount,
                    outstanding: *amount,
                    accrual_at: *posted_at,
                    due_at,
                });
            }
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::PaymentAllocation(_),
                balance_type,
                amount,
                ..
            } => match balance_type {
                BalanceUpdatedType::Disbursal => due_and_outstanding_disbursed -= *amount,
                BalanceUpdatedType::InterestAccrual => due_and_outstanding_interest += *amount,
            },
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

    let maturity_date = terms.duration.maturity_date(activated_at);

    let mut next_interest_period = if let Some(last_interest_payment) = last_interest_accrual_at {
        terms
            .accrual_cycle_interval
            .period_from(last_interest_payment)
            .next()
            .truncate(maturity_date)
    } else {
        terms
            .accrual_cycle_interval
            .period_from(activated_at)
            .truncate(maturity_date)
    };

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

            next_interest_period = period.next().truncate(maturity_date);
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
        accrual_at: maturity_date,
        due_at: maturity_date,
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
            .interest_due_duration(InterestDuration::Days(0))
            .accrual_cycle_interval(InterestInterval::EndOfMonth)
            .accrual_interval(InterestInterval::EndOfDay)
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
        let disbursal_obligation_id = ObligationId::new();
        let first_interest_idx = InterestAccrualCycleIdx::FIRST;
        let first_interest_posted_at = end_of_month(activated_at);
        let interest_obligation_id = ObligationId::new();
        vec![
            CreditFacilityEvent::Initialized {
                id: credit_facility_id,
                customer_id: CustomerId::new(),
                account_ids: CreditFacilityAccountIds::new(),
                amount: UsdCents::from(1_000_000),
                terms: Box::new(terms()),
                audit_info: dummy_audit_info(),
                disbursal_credit_account_id: CalaAccountId::new(),
                approval_process_id: ApprovalProcessId::new(),
            },
            CreditFacilityEvent::Activated {
                ledger_tx_id: LedgerTxId::new(),
                activated_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(disbursal_obligation_id),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::Disbursal,
                amount: UsdCents::from(1000),
                updated_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::InterestAccrualCycleConcluded {
                idx: first_interest_idx,
                tx_id: LedgerTxId::new(),
                obligation_id: interest_obligation_id,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(interest_obligation_id),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(2),
                updated_at: first_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::PaymentAllocation(PaymentAllocationId::new()),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(2),
                updated_at: first_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
        ]
    }

    #[test]
    fn no_interest_repayment() {
        let credit_facility_id = CreditFacilityId::new();
        let activated_at = default_activated_at();
        let disbursal_obligation_id = ObligationId::new();

        let events = vec![
            CreditFacilityEvent::Initialized {
                id: credit_facility_id,
                customer_id: CustomerId::new(),
                account_ids: CreditFacilityAccountIds::new(),
                amount: UsdCents::from(1_000_000),
                terms: Box::new(terms()),
                audit_info: dummy_audit_info(),
                disbursal_credit_account_id: CalaAccountId::new(),
                approval_process_id: ApprovalProcessId::new(),
            },
            CreditFacilityEvent::Activated {
                ledger_tx_id: LedgerTxId::new(),
                activated_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(disbursal_obligation_id),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::Disbursal,
                amount: UsdCents::from(1000),
                updated_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
        ];

        let repayment_plan = super::project(events.iter());

        let n_existing_interest_accruals = 0;
        let n_future_interest_accruals = 3;
        let n_principal_accruals = 1;
        assert_eq!(
            repayment_plan.len(),
            n_existing_interest_accruals + n_future_interest_accruals + n_principal_accruals
        );

        for item in repayment_plan.iter().take(3) {
            match item {
                CreditFacilityRepaymentInPlan::Interest(interest) => {
                    assert_eq!(interest.status, RepaymentStatus::Upcoming);
                }
                _ => panic!("Expected first 3 elements to be Interest"),
            }
        }

        match &repayment_plan[3] {
            CreditFacilityRepaymentInPlan::Disbursal(principal) => {
                assert_eq!(principal.status, RepaymentStatus::Upcoming);
            }
            _ => panic!("Expected fourth element to be Disbursal"),
        }
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
        let first_interest_posted_at = end_of_month(default_activated_at());
        let second_interest_idx = InterestAccrualCycleIdx::FIRST.next();
        let second_interest_posted_at =
            end_of_month(first_interest_posted_at + chrono::Duration::days(1));
        let obligation_id = ObligationId::new();
        events.extend([
            CreditFacilityEvent::InterestAccrualCycleConcluded {
                idx: second_interest_idx,
                tx_id: LedgerTxId::new(),
                obligation_id,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(obligation_id),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(12),
                updated_at: second_interest_posted_at,
                audit_info: dummy_audit_info(),
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
            }
            _ => panic!("Expected second element to be Interest"),
        }
    }

    #[test]
    fn partial_interest_payment() {
        let mut events = happy_credit_facility_events();
        let first_interest_posted_at = end_of_month(default_activated_at());
        let second_interest_idx = InterestAccrualCycleIdx::FIRST.next();
        let second_interest_posted_at =
            end_of_month(first_interest_posted_at + chrono::Duration::days(1));
        let obligation_id = ObligationId::new();
        events.extend([
            CreditFacilityEvent::InterestAccrualCycleConcluded {
                idx: second_interest_idx,
                tx_id: LedgerTxId::new(),
                obligation_id,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(obligation_id),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(12),
                updated_at: second_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::PaymentAllocation(PaymentAllocationId::new()),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(2),
                updated_at: second_interest_posted_at,
                audit_info: dummy_audit_info(),
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
        let second_disbursal_at = default_activated_at() + chrono::Duration::days(1);
        let disbursal_obligation_id = ObligationId::new();
        events.extend([CreditFacilityEvent::BalanceUpdated {
            source: BalanceUpdatedSource::Obligation(disbursal_obligation_id),
            ledger_tx_id: LedgerTxId::new(),
            balance_type: BalanceUpdatedType::Disbursal,
            amount: UsdCents::from(2000),
            updated_at: second_disbursal_at,
            audit_info: dummy_audit_info(),
        }]);
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
        let first_interest_posted_at = end_of_month(default_activated_at());
        let second_interest_idx = InterestAccrualCycleIdx::FIRST.next();
        let second_interest_posted_at =
            end_of_month(first_interest_posted_at + chrono::Duration::days(1));
        let second_obligation_id = ObligationId::new();
        let third_interest_posted_at =
            end_of_month(second_interest_posted_at + chrono::Duration::days(1));
        let third_obligation_id = ObligationId::new();
        events.extend([
            CreditFacilityEvent::InterestAccrualCycleConcluded {
                idx: second_interest_idx,
                tx_id: LedgerTxId::new(),
                obligation_id: second_obligation_id,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(second_obligation_id),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(12),
                updated_at: second_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::PaymentAllocation(PaymentAllocationId::new()),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(12),
                updated_at: second_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::InterestAccrualCycleConcluded {
                idx: second_interest_idx.next(),
                tx_id: LedgerTxId::new(),
                obligation_id: ObligationId::new(),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(third_obligation_id),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(6),
                updated_at: third_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::PaymentAllocation(PaymentAllocationId::new()),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::Disbursal,
                amount: UsdCents::from(100),
                updated_at: third_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::PaymentAllocation(PaymentAllocationId::new()),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(6),
                updated_at: third_interest_posted_at,
                audit_info: dummy_audit_info(),
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
        let first_interest_posted_at = end_of_month(default_activated_at());
        let second_interest_idx = InterestAccrualCycleIdx::FIRST.next();
        let second_interest_posted_at =
            end_of_month(first_interest_posted_at + chrono::Duration::days(1));
        let second_obligation_id = ObligationId::new();
        let third_interest_posted_at =
            end_of_month(second_interest_posted_at + chrono::Duration::days(1));
        let third_obligation_id = ObligationId::new();
        events.extend([
            CreditFacilityEvent::InterestAccrualCycleConcluded {
                idx: second_interest_idx,
                tx_id: LedgerTxId::new(),
                obligation_id: second_obligation_id,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(second_obligation_id),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(12),
                updated_at: second_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::PaymentAllocation(PaymentAllocationId::new()),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(12),
                updated_at: second_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::InterestAccrualCycleConcluded {
                idx: second_interest_idx.next(),
                tx_id: LedgerTxId::new(),
                obligation_id: ObligationId::new(),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(third_obligation_id),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(6),
                updated_at: third_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::PaymentAllocation(PaymentAllocationId::new()),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::Disbursal,
                amount: UsdCents::from(1000),
                updated_at: third_interest_posted_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::PaymentAllocation(PaymentAllocationId::new()),
                ledger_tx_id: LedgerTxId::new(),
                balance_type: BalanceUpdatedType::InterestAccrual,
                amount: UsdCents::from(6),
                updated_at: third_interest_posted_at,
                audit_info: dummy_audit_info(),
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
