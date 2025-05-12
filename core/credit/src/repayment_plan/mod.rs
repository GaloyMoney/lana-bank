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

    fn disbursed_outstanding(&self) -> UsdCents {
        self.entries
            .iter()
            .filter_map(|entry| match entry {
                CreditFacilityRepaymentPlanEntry::Disbursal(data) => Some(data.outstanding),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, outstanding| acc + outstanding)
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

    fn planned_interest_accruals(&self) -> Vec<CreditFacilityRepaymentPlanEntry> {
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

        let mut entries = vec![];
        while let Some(period) = next_interest_period {
            let interest = terms
                .annual_rate
                .interest_for_time_period(self.disbursed_outstanding(), period.days());

            entries.push(CreditFacilityRepaymentPlanEntry::Interest(
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

        entries
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
                created_at,
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
                    recorded_at: *created_at,
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
                        _ => unreachable!(),
                    };
                } else {
                    return false;
                }
            }

            _ => return false,
        };

        self.entries = if !existing_obligations.is_empty() {
            existing_obligations
        } else {
            self.planned_disbursals()
        };

        self.entries.extend(self.planned_interest_accruals());

        self.entries.sort();

        true
    }
}
