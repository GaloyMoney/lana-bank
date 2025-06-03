mod entry;
pub mod error;
mod repo;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use outbox::EventSequence;

use crate::{event::CoreCreditEvent, primitives::*, terms::TermValues};

pub use entry::*;
pub use repo::RepaymentPlanRepo;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreditFacilityRepaymentPlan {
    facility_amount: UsdCents,
    terms: Option<TermValues>,
    activated_at: Option<DateTime<Utc>>,
    last_interest_accrual_at: Option<DateTime<Utc>>,
    last_updated_on_sequence: EventSequence,

    pub entries: Vec<CreditFacilityRepaymentPlanEntry>,
}

impl CreditFacilityRepaymentPlan {
    fn activated_at(&self) -> DateTime<Utc> {
        self.activated_at.unwrap_or(crate::time::now())
    }

    fn existing_obligations(&self) -> Vec<CreditFacilityRepaymentPlanEntry> {
        self.entries
            .iter()
            .filter_map(|entry| match entry {
                CreditFacilityRepaymentPlanEntry::Disbursal(data)
                | CreditFacilityRepaymentPlanEntry::Interest(data)
                    if data.id.is_some() =>
                {
                    Some(*entry)
                }
                _ => None,
            })
            .collect()
    }

    fn planned_disbursals(&self) -> Vec<CreditFacilityRepaymentPlanEntry> {
        let terms = self.terms.expect("Missing FacilityCreated event");
        let facility_amount = self.facility_amount;
        let structuring_fee = terms.one_time_fee_rate.apply(facility_amount);

        let activated_at = self.activated_at();
        let maturity_date = terms.duration.maturity_date(activated_at);

        vec![
            CreditFacilityRepaymentPlanEntry::Disbursal(ObligationDataForEntry {
                id: None,
                status: RepaymentStatus::Upcoming,

                initial: structuring_fee,
                outstanding: structuring_fee,

                due_at: maturity_date,
                recorded_at: activated_at,
                effective: activated_at.date_naive(),
            }),
            CreditFacilityRepaymentPlanEntry::Disbursal(ObligationDataForEntry {
                id: None,
                status: RepaymentStatus::Upcoming,

                initial: facility_amount,
                outstanding: facility_amount,

                due_at: maturity_date,
                recorded_at: activated_at,
                effective: activated_at.date_naive(),
            }),
        ]
    }

    fn planned_interest_accruals(
        &self,
        updated_entries: &[CreditFacilityRepaymentPlanEntry],
    ) -> Vec<CreditFacilityRepaymentPlanEntry> {
        let terms = self.terms.expect("Missing FacilityCreated event");
        let activated_at = self.activated_at();

        let maturity_date = terms.duration.maturity_date(activated_at);
        let mut next_interest_period =
            if let Some(last_interest_payment) = self.last_interest_accrual_at {
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

        let disbursed_outstanding = updated_entries
            .iter()
            .filter_map(|entry| match entry {
                CreditFacilityRepaymentPlanEntry::Disbursal(data) => Some(data.outstanding),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, outstanding| acc + outstanding);

        let mut planned_interest_entries = vec![];
        while let Some(period) = next_interest_period {
            let interest = terms
                .annual_rate
                .interest_for_time_period(disbursed_outstanding, period.days());

            planned_interest_entries.push(CreditFacilityRepaymentPlanEntry::Interest(
                ObligationDataForEntry {
                    id: None,
                    status: RepaymentStatus::Upcoming,
                    initial: interest,
                    outstanding: interest,

                    due_at: period.end,
                    recorded_at: period.end,
                    effective: period.end.date_naive(),
                },
            ));

            next_interest_period = period.next().truncate(maturity_date);
        }

        planned_interest_entries
    }

    pub(super) fn process_event(
        &mut self,
        sequence: EventSequence,
        event: &CoreCreditEvent,
    ) -> bool {
        self.last_updated_on_sequence = sequence;

        let mut existing_obligations = self.existing_obligations();

        match event {
            CoreCreditEvent::FacilityCreated { terms, amount, .. } => {
                self.terms = Some(*terms);
                self.facility_amount = *amount;
            }
            CoreCreditEvent::FacilityActivated { activated_at, .. } => {
                self.activated_at = Some(*activated_at);
            }
            CoreCreditEvent::ObligationCreated {
                id,
                obligation_type,
                amount,
                due_at,
                recorded_at,
                effective,
                ..
            } => {
                let data = ObligationDataForEntry {
                    id: Some(*id),
                    status: RepaymentStatus::NotYetDue,

                    initial: *amount,
                    outstanding: *amount,

                    due_at: *due_at,
                    recorded_at: *recorded_at,
                    effective: *effective,
                };

                let effective = EffectiveDate::from(*effective);
                let entry = match obligation_type {
                    ObligationType::Disbursal => CreditFacilityRepaymentPlanEntry::Disbursal(data),
                    ObligationType::Interest => {
                        self.last_interest_accrual_at = Some(effective.end_of_day());
                        CreditFacilityRepaymentPlanEntry::Interest(data)
                    }
                };

                existing_obligations.push(entry);
            }
            CoreCreditEvent::FacilityRepaymentRecorded {
                obligation_id,
                amount,
                ..
            } => {
                if let Some(data) = existing_obligations.iter_mut().find_map(|entry| {
                    let data = match entry {
                        CreditFacilityRepaymentPlanEntry::Disbursal(data)
                        | CreditFacilityRepaymentPlanEntry::Interest(data) => data,
                    };

                    (data.id == Some(*obligation_id)).then_some(data)
                }) {
                    data.outstanding -= *amount;
                } else {
                    return false;
                }
            }
            CoreCreditEvent::ObligationDue {
                id: obligation_id, ..
            }
            | CoreCreditEvent::ObligationOverdue {
                id: obligation_id, ..
            }
            | CoreCreditEvent::ObligationCompleted {
                id: obligation_id, ..
            } => {
                if let Some(data) = existing_obligations.iter_mut().find_map(|entry| {
                    let data = match entry {
                        CreditFacilityRepaymentPlanEntry::Disbursal(data)
                        | CreditFacilityRepaymentPlanEntry::Interest(data) => data,
                    };

                    (data.id == Some(*obligation_id)).then_some(data)
                }) {
                    data.status = match event {
                        CoreCreditEvent::ObligationDue { .. } => RepaymentStatus::Due,
                        CoreCreditEvent::ObligationOverdue { .. } => RepaymentStatus::Overdue,
                        CoreCreditEvent::ObligationCompleted { .. } => RepaymentStatus::Paid,
                        _ => unreachable!(),
                    };
                } else {
                    return false;
                }
            }

            _ => return false,
        };

        let updated_entries = if !existing_obligations.is_empty() {
            existing_obligations
        } else {
            self.planned_disbursals()
        };

        let planned_interest_entries = self.planned_interest_accruals(&updated_entries);

        self.entries = updated_entries
            .into_iter()
            .chain(planned_interest_entries)
            .collect();
        self.entries.sort();

        true
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use crate::terms::{FacilityDuration, InterestInterval, ObligationDuration, OneTimeFeeRatePct};

    use super::*;

    #[derive(Debug, Default, PartialEq, Eq)]
    struct EntriesCount {
        interest_unpaid: usize,
        interest_paid: usize,
        interest_upcoming: usize,
        disbursals_unpaid: usize,
        disbursals_paid: usize,
        disbursals_upcoming: usize,
    }

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(FacilityDuration::Months(3))
            .interest_due_duration(ObligationDuration::Days(0))
            .obligation_overdue_duration(None)
            .obligation_liquidation_duration(None)
            .accrual_cycle_interval(InterestInterval::EndOfMonth)
            .accrual_interval(InterestInterval::EndOfDay)
            .one_time_fee_rate(OneTimeFeeRatePct::new(5))
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term")
    }

    fn default_start_date() -> DateTime<Utc> {
        "2021-01-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap()
    }

    fn default_start_date_with_days(days: i64) -> DateTime<Utc> {
        "2021-01-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap() + chrono::Duration::days(days)
    }

    fn default_facility_amount() -> UsdCents {
        UsdCents::from(1_000_000_00)
    }

    fn initial_plan() -> CreditFacilityRepaymentPlan {
        let mut plan = CreditFacilityRepaymentPlan::default();
        plan.process_event(
            Default::default(),
            &CoreCreditEvent::FacilityCreated {
                id: CreditFacilityId::new(),
                terms: default_terms(),
                amount: default_facility_amount(),
                created_at: default_start_date(),
            },
        );

        plan
    }

    fn process_events(plan: &mut CreditFacilityRepaymentPlan, events: Vec<CoreCreditEvent>) {
        for event in events {
            plan.process_event(Default::default(), &event);
        }
    }

    fn count_entries(plan: &CreditFacilityRepaymentPlan) -> EntriesCount {
        let mut res = EntriesCount::default();

        for entry in plan.entries.iter() {
            match entry {
                CreditFacilityRepaymentPlanEntry::Disbursal(ObligationDataForEntry {
                    status: RepaymentStatus::Upcoming,
                    ..
                }) => res.disbursals_upcoming += 1,
                CreditFacilityRepaymentPlanEntry::Disbursal(ObligationDataForEntry {
                    status: RepaymentStatus::Paid,
                    ..
                }) => res.disbursals_paid += 1,
                CreditFacilityRepaymentPlanEntry::Disbursal(ObligationDataForEntry { .. }) => {
                    res.disbursals_unpaid += 1
                }
                CreditFacilityRepaymentPlanEntry::Interest(ObligationDataForEntry {
                    status: RepaymentStatus::Upcoming,
                    ..
                }) => res.interest_upcoming += 1,
                CreditFacilityRepaymentPlanEntry::Interest(ObligationDataForEntry {
                    status: RepaymentStatus::Paid,
                    ..
                }) => res.interest_paid += 1,
                CreditFacilityRepaymentPlanEntry::Interest(ObligationDataForEntry { .. }) => {
                    res.interest_unpaid += 1
                }
            }
        }

        res
    }

    #[test]
    fn facility_created() {
        let plan = initial_plan();
        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 0,
                interest_paid: 0,
                interest_upcoming: 4,
                disbursals_unpaid: 0,
                disbursals_paid: 0,
                disbursals_upcoming: 2,
            }
        );
    }

    #[test]
    fn with_first_disbursal_obligation_created() {
        let mut plan = initial_plan();

        let recorded_at = default_start_date();
        let events = vec![
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: default_start_date(),
                overdue_at: None,
                liquidation_at: None,
                recorded_at,
                effective: recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 0,
                interest_paid: 0,
                interest_upcoming: 4,
                disbursals_unpaid: 1,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );
    }

    #[test]
    fn with_first_interest_obligation_created() {
        let mut plan = initial_plan();

        let disbursal_recorded_at = default_start_date();
        let interest_recorded_at = default_start_date_with_days(30);
        let events = vec![
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: disbursal_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: disbursal_recorded_at,
                effective: disbursal_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: interest_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 1,
                interest_paid: 0,
                interest_upcoming: 3,
                disbursals_unpaid: 1,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );
    }

    #[test]
    fn with_first_interest_partial_payment() {
        let interest_obligation_id = ObligationId::new();

        let mut plan = initial_plan();

        let disbursal_recorded_at = default_start_date();
        let interest_recorded_at = default_start_date_with_days(30);
        let events = vec![
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: disbursal_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: disbursal_recorded_at,
                effective: disbursal_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: interest_obligation_id,
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: interest_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
            CoreCreditEvent::FacilityRepaymentRecorded {
                credit_facility_id: CreditFacilityId::new(),
                obligation_id: interest_obligation_id,
                obligation_type: ObligationType::Interest,
                payment_id: PaymentAllocationId::new(),
                amount: UsdCents::from(400_00),
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 1,
                interest_paid: 0,
                interest_upcoming: 3,
                disbursals_unpaid: 1,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );

        let interest_entry_outstanding = plan
            .entries
            .iter()
            .find_map(|e| match e {
                CreditFacilityRepaymentPlanEntry::Interest(ObligationDataForEntry {
                    id,
                    outstanding,
                    ..
                }) if id.is_some() => Some(outstanding),
                _ => None,
            })
            .unwrap();
        assert_eq!(*interest_entry_outstanding, UsdCents::from(600_00));
    }

    #[test]
    fn with_first_interest_paid() {
        let interest_obligation_id = ObligationId::new();

        let mut plan = initial_plan();

        let disbursal_recorded_at = default_start_date();
        let interest_recorded_at = default_start_date_with_days(30);
        let events = vec![
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: disbursal_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: disbursal_recorded_at,
                effective: disbursal_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: interest_obligation_id,
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: interest_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
            CoreCreditEvent::FacilityRepaymentRecorded {
                credit_facility_id: CreditFacilityId::new(),
                obligation_id: interest_obligation_id,
                obligation_type: ObligationType::Interest,
                payment_id: PaymentAllocationId::new(),
                amount: UsdCents::from(1_000_00),
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 1,
                interest_paid: 0,
                interest_upcoming: 3,
                disbursals_unpaid: 1,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );

        let (outstanding, status) = plan
            .entries
            .iter()
            .find_map(|e| match e {
                CreditFacilityRepaymentPlanEntry::Interest(ObligationDataForEntry {
                    id,
                    outstanding,
                    status,
                    ..
                }) if id.is_some() => Some((outstanding, status)),
                _ => None,
            })
            .unwrap();
        assert_eq!(*outstanding, UsdCents::ZERO);
        assert_ne!(*status, RepaymentStatus::Paid);

        plan.process_event(
            Default::default(),
            &CoreCreditEvent::ObligationCompleted {
                id: interest_obligation_id,
                credit_facility_id: CreditFacilityId::new(),
            },
        );
        let interest_entry_status = plan
            .entries
            .iter()
            .find_map(|e| match e {
                CreditFacilityRepaymentPlanEntry::Interest(ObligationDataForEntry {
                    id,
                    status,
                    ..
                }) if id.is_some() => Some(status),
                _ => None,
            })
            .unwrap();
        assert_eq!(*interest_entry_status, RepaymentStatus::Paid);
    }

    #[test]
    fn with_all_interest_obligations_created() {
        let mut plan = initial_plan();

        let disbursal_recorded_at = default_start_date();
        let interest_1_recorded_at = default_start_date_with_days(30);
        let interest_2_recorded_at = default_start_date_with_days(30 + 28);
        let interest_3_recorded_at = default_start_date_with_days(30 + 28 + 31);
        let interest_4_recorded_at = default_start_date_with_days(30 + 28 + 31 + 1);
        let events = vec![
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: disbursal_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: disbursal_recorded_at,
                effective: disbursal_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: interest_1_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: interest_1_recorded_at,
                effective: interest_1_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: interest_2_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: interest_2_recorded_at,
                effective: interest_2_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: interest_3_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: interest_3_recorded_at,
                effective: interest_3_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(33_00),
                due_at: interest_4_recorded_at,
                overdue_at: None,
                liquidation_at: None,
                recorded_at: interest_4_recorded_at,
                effective: interest_4_recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 4,
                interest_paid: 0,
                interest_upcoming: 0,
                disbursals_unpaid: 1,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );
    }
}
