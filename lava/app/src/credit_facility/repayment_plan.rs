use chrono::{DateTime, Utc};

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
    pub amount: UsdCents,
    pub accrual_at: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreditFacilityRepaymentInPlan {
    Disbursal(RepaymentInPlan),
    // TODO: Interest
}

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a CreditFacilityEvent>,
) -> Vec<CreditFacilityRepaymentInPlan> {
    let mut terms = None;
    let mut activated_at = None;
    let mut amounts = std::collections::HashMap::new();
    let mut net_disbursed = UsdCents::ZERO;

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
                amounts.insert(*idx, *amount);
            }
            CreditFacilityEvent::DisbursalConcluded { idx, .. } => {
                if let Some(amount) = amounts.remove(idx) {
                    net_disbursed += amount;
                }
            }
            CreditFacilityEvent::PaymentRecorded {
                disbursal_amount, ..
            } => {
                net_disbursed -= *disbursal_amount;
            }
            _ => {}
        }
    }

    let terms = terms.expect("Initialized event not found");
    let activated_at = match activated_at {
        Some(time) => time,
        None => return Vec::new(),
    };

    let expiry_date = terms.duration.expiration_date(activated_at);
    let status = if net_disbursed == UsdCents::ZERO {
        RepaymentStatus::Paid
    } else {
        RepaymentStatus::Upcoming
    };

    vec![CreditFacilityRepaymentInPlan::Disbursal(RepaymentInPlan {
        status,
        amount: net_disbursed,
        accrual_at: activated_at,
        due_at: expiry_date,
    })]
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use crate::{
        credit_facility::CreditFacilityAccountIds, ledger::customer::*, primitives::*, terms::*,
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

    fn happy_credit_facility_events() -> Vec<CreditFacilityEvent> {
        let credit_facility_id = CreditFacilityId::new();
        vec![
            CreditFacilityEvent::Initialized {
                id: credit_facility_id,
                customer_id: CustomerId::new(),
                account_ids: CreditFacilityAccountIds::new(),
                customer_account_ids: CustomerLedgerAccountIds::new(),
                facility: UsdCents::from(1_000_000),
                terms: terms(),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::Approved {
                tx_id: LedgerTxId::new(),
                recorded_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalInitiated {
                approval_process_id: ApprovalProcessId::new(),
                disbursal_id: DisbursalId::new(),
                idx: DisbursalIdx::FIRST,
                amount: UsdCents::from(1000),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalConcluded {
                idx: DisbursalIdx::FIRST,
                tx_id: Some(LedgerTxId::new()),
                recorded_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::PaymentRecorded {
                tx_id: LedgerTxId::new(),
                disbursal_amount: UsdCents::from(100),
                interest_amount: UsdCents::from(10),
                audit_info: dummy_audit_info(),
                tx_ref: LedgerTxId::new().to_string(),
                recorded_in_ledger_at: Utc::now(),
            },
        ]
    }

    #[test]
    fn generates_disbursal_minus_paid_in_plan() {
        let events = happy_credit_facility_events();
        let repayment_plan = super::project(events.iter());

        assert_eq!(repayment_plan.len(), 1);
        match &repayment_plan[0] {
            CreditFacilityRepaymentInPlan::Disbursal(repayment) => {
                assert_eq!(repayment.status, RepaymentStatus::Upcoming);
                assert_eq!(repayment.amount, UsdCents::from(900));
            }
        }
    }
}
