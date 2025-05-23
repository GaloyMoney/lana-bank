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
                overdue_at: None,
                defaulted_at: None,
                recorded_at: activated_at,
            }),
            CreditFacilityRepaymentPlanEntry::Disbursal(ObligationDataForEntry {
                id: None,
                status: RepaymentStatus::Upcoming,

                initial: facility_amount,
                outstanding: facility_amount,

                due_at: maturity_date,
                overdue_at: None,
                defaulted_at: None,
                recorded_at: activated_at,
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
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: period.end,
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
                overdue_at,
                defaulted_at,
                recorded_at,
                ..
            } => {
                let data = ObligationDataForEntry {
                    id: Some(*id),
                    status: RepaymentStatus::NotYetDue,

                    initial: *amount,
                    outstanding: *amount,

                    due_at: *due_at,
                    overdue_at: *overdue_at,
                    defaulted_at: *defaulted_at,
                    recorded_at: *recorded_at,
                };
                let entry = match obligation_type {
                    ObligationType::Disbursal => CreditFacilityRepaymentPlanEntry::Disbursal(data),
                    ObligationType::Interest => {
                        self.last_interest_accrual_at = Some(data.recorded_at);
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
            | CoreCreditEvent::ObligationDefaulted {
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
                        CoreCreditEvent::ObligationDefaulted { .. } => RepaymentStatus::Defaulted,
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

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(FacilityDuration::Months(3))
            .interest_due_duration(ObligationDuration::Days(0))
            .obligation_overdue_duration(None)
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

    #[test]
    fn planned_disbursals_returns_expected_number_of_entries() {
        assert_eq!(initial_plan().planned_disbursals().len(), 2);
    }

    #[test]
    fn planned_interest_accruals_returns_expected_number_of_entries() {
        let mut plan = initial_plan();
        assert_eq!(plan.planned_interest_accruals(&plan.entries).len(), 4);

        plan.process_event(
            Default::default(),
            &CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
        );
        plan.process_event(
            Default::default(),
            &CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: default_start_date(),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: default_start_date(),
            },
        );
        plan.process_event(
            Default::default(),
            &CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: default_start_date() + chrono::Duration::days(30),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: default_start_date() + chrono::Duration::days(30),
            },
        );
        assert_eq!(plan.planned_interest_accruals(&plan.entries).len(), 3);
    }
}
