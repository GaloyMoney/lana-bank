use chrono::{DateTime, Utc};

use crate::primitives::*;

use super::CreditFacilityEvent;

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a CreditFacilityEvent>,
) -> Vec<CreditFacilityQuoteEntry> {
    let mut entries = vec![];

    let mut terms = None;
    let mut amount = None;
    let mut is_activated = None;

    for event in events {
        match event {
            CreditFacilityEvent::Initialized {
                terms: t,
                amount: a,
                ..
            } => {
                terms = Some(*t);
                amount = Some(*a);
            }

            CreditFacilityEvent::Activated { .. } => {
                is_activated = Some(true);
            }

            _ => {}
        }
    }

    if is_activated.unwrap_or(false) {
        return entries;
    }

    let terms = terms.expect("Facility was not Initialized");
    let facility_amount = amount.expect("Facility was not Initialized");
    let structuring_fee = terms.one_time_fee_rate.apply(facility_amount);

    let activated_at = crate::time::now();
    let maturity_date = terms.duration.maturity_date(activated_at);

    entries.extend([
        CreditFacilityQuoteEntry::Fee(ObligationDataForQuoteEntry {
            outstanding: structuring_fee,
            due_at: activated_at,
        }),
        CreditFacilityQuoteEntry::Disbursal(ObligationDataForQuoteEntry {
            outstanding: facility_amount,
            due_at: activated_at,
        }),
    ]);

    let last_interest_accrual_at = None;
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

    while let Some(period) = next_interest_period {
        let interest = terms
            .annual_rate
            .interest_for_time_period(facility_amount, period.days());

        entries.push(CreditFacilityQuoteEntry::Interest(
            ObligationDataForQuoteEntry {
                outstanding: interest,
                due_at: period.end,
            },
        ));

        next_interest_period = period.next().truncate(maturity_date);
    }

    entries.sort();

    entries
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObligationDataForQuoteEntry {
    pub outstanding: UsdCents,
    pub due_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditFacilityQuoteEntry {
    Disbursal(ObligationDataForQuoteEntry),
    Interest(ObligationDataForQuoteEntry),
    Fee(ObligationDataForQuoteEntry),
}

impl PartialOrd for CreditFacilityQuoteEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CreditFacilityQuoteEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_due_at = match self {
            CreditFacilityQuoteEntry::Disbursal(o) => o.due_at,
            CreditFacilityQuoteEntry::Fee(o) => o.due_at,
            CreditFacilityQuoteEntry::Interest(o) => o.due_at,
        };

        let other_due_at = match other {
            CreditFacilityQuoteEntry::Disbursal(o) => o.due_at,
            CreditFacilityQuoteEntry::Fee(o) => o.due_at,
            CreditFacilityQuoteEntry::Interest(o) => o.due_at,
        };

        self_due_at.cmp(&other_due_at)
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use audit::{AuditEntryId, AuditInfo};

    use crate::{terms::*, CreditFacilityAccountIds};

    use super::*;

    fn terms(months: u32) -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(months))
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

    #[test]
    fn quote() {
        let facility_amount = UsdCents::from(1_000_000_00);
        let months = 2;
        let events = vec![CreditFacilityEvent::Initialized {
            id: CreditFacilityId::new(),
            customer_id: CustomerId::new(),
            account_ids: CreditFacilityAccountIds::new(),
            amount: facility_amount,
            terms: terms(months),
            audit_info: dummy_audit_info(),
            disbursal_credit_account_id: CalaAccountId::new(),
            approval_process_id: ApprovalProcessId::new(),
            collateral_id: CollateralId::new(),
            ledger_tx_id: LedgerTxId::new(),
        }];
        let mut quote = project(events.iter());

        let structuring_fee_position = quote
            .iter()
            .position(|e| match e {
                CreditFacilityQuoteEntry::Fee(ObligationDataForQuoteEntry { .. }) => true,
                _ => false,
            })
            .unwrap();
        assert_eq!(structuring_fee_position, 0);
        quote.remove(structuring_fee_position);

        let facility_disbursal_position = quote
            .iter()
            .position(|e| match e {
                CreditFacilityQuoteEntry::Disbursal(ObligationDataForQuoteEntry {
                    outstanding,
                    ..
                }) if *outstanding == facility_amount => true,
                _ => false,
            })
            .unwrap();
        assert_eq!(facility_disbursal_position, 0);
        quote.remove(facility_disbursal_position);

        assert!(quote.iter().all(|e| match e {
            CreditFacilityQuoteEntry::Interest(_) => true,
            _ => false,
        }));

        let n_interest: u32 = quote.len().try_into().unwrap();
        assert!(n_interest >= months);
        assert!(n_interest <= months + 1);
    }
}
