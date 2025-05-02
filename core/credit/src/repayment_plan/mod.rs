mod entry;
pub mod error;
mod repo;
mod values;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use outbox::EventSequence;

use crate::{event::CoreCreditEvent, primitives::*, terms::TermValues};

pub use entry::*;
pub use repo::RepaymentPlanRepo;
pub use values::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreditFacilityRepaymentPlan {
    terms: Option<TermValues>,
    activated_at: DateTime<Utc>,

    existing_obligations: Vec<RecordedObligationInPlan>,
    last_updated_on_sequence: EventSequence,

    pub entries: Vec<CreditFacilityRepaymentPlanEntry>,
}

impl CreditFacilityRepaymentPlan {
    fn disbursed_outstanding(&self) -> UsdCents {
        self.existing_obligations
            .iter()
            .filter_map(|obligation| {
                let ObligationInPlan {
                    obligation_type,
                    outstanding,
                    ..
                } = obligation.values;
                if obligation_type == ObligationType::Disbursal {
                    Some(outstanding)
                } else {
                    None
                }
            })
            .fold(UsdCents::ZERO, |acc, outstanding| acc + outstanding)
    }

    fn update_upcoming(&mut self) {
        self.entries = self
            .existing_obligations
            .iter()
            .map(CreditFacilityRepaymentPlanEntry::from)
            .collect::<Vec<_>>();
        let outstanding = self.disbursed_outstanding();
        if outstanding.is_zero() {
            return;
        }

        let terms = self.terms.expect("Missing FacilityCreated event");
        let maturity_date = terms.duration.maturity_date(self.activated_at);
        let last_interest_accrual_at = self.existing_obligations.iter().rev().find_map(|o| {
            if o.values.obligation_type == ObligationType::Interest {
                Some(o.values.recorded_at)
            } else {
                None
            }
        });
        let mut next_interest_period = if let Some(last_interest_payment) = last_interest_accrual_at
        {
            terms
                .accrual_cycle_interval
                .period_from(last_interest_payment)
                .next()
                .truncate(maturity_date)
        } else {
            terms
                .accrual_cycle_interval
                .period_from(self.activated_at)
                .truncate(maturity_date)
        };

        while let Some(period) = next_interest_period {
            let interest = terms
                .annual_rate
                .interest_for_time_period(outstanding, period.days());

            self.entries
                .push(CreditFacilityRepaymentPlanEntry::Interest(
                    ObligationDataForEntry {
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

        self.entries.sort();
    }

    pub(super) fn process_event(
        &mut self,
        sequence: EventSequence,
        event: &CoreCreditEvent,
    ) -> bool {
        self.last_updated_on_sequence = sequence;
        let plan_updated = match event {
            CoreCreditEvent::FacilityCreated { terms, .. } => {
                self.terms = Some(*terms);

                true
            }
            CoreCreditEvent::FacilityActivated { activated_at, .. } => {
                self.activated_at = *activated_at;

                true
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
                self.existing_obligations.push(RecordedObligationInPlan {
                    obligation_id: *id,
                    values: ObligationInPlan {
                        obligation_type: *obligation_type,
                        status: RepaymentStatus::NotYetDue,

                        initial: *amount,
                        outstanding: *amount,

                        due_at: *due_at,
                        overdue_at: *overdue_at,
                        defaulted_at: *defaulted_at,
                        recorded_at: *created_at,
                    },
                });
                true
            }
            CoreCreditEvent::FacilityRepaymentRecorded {
                obligation_id,
                amount,
                ..
            } => {
                if let Some(r) = self
                    .existing_obligations
                    .iter_mut()
                    .find(|r| r.obligation_id == *obligation_id)
                {
                    r.values.outstanding -= *amount;
                    true
                } else {
                    false
                }
            }
            CoreCreditEvent::ObligationDue {
                id: obligation_id, ..
            } => {
                if let Some(r) = self
                    .existing_obligations
                    .iter_mut()
                    .find(|r| r.obligation_id == *obligation_id)
                {
                    r.values.status = RepaymentStatus::Due;
                    true
                } else {
                    false
                }
            }
            CoreCreditEvent::ObligationOverdue {
                id: obligation_id, ..
            } => {
                if let Some(r) = self
                    .existing_obligations
                    .iter_mut()
                    .find(|r| r.obligation_id == *obligation_id)
                {
                    r.values.status = RepaymentStatus::Overdue;
                    true
                } else {
                    false
                }
            }
            CoreCreditEvent::ObligationDefaulted {
                id: obligation_id, ..
            } => {
                if let Some(r) = self
                    .existing_obligations
                    .iter_mut()
                    .find(|r| r.obligation_id == *obligation_id)
                {
                    r.values.status = RepaymentStatus::Defaulted;
                    true
                } else {
                    false
                }
            }

            _ => false,
        };

        if !plan_updated {
            false
        } else if self.existing_obligations.is_empty() {
            true
        } else {
            self.update_upcoming();
            true
        }
    }
}
