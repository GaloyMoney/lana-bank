use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    entity::{Entity, EntityError, EntityEvent, EntityEvents},
    primitives::{AuditInfo, CreditFacilityId, InterestAccrualId, InterestAccrualIdx, UsdCents},
    terms::{InterestPeriod, TermValues},
};

use super::InterestAccrualError;

pub struct OutstandingForPeriod {
    period: InterestPeriod,
    amount: UsdCents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InterestAccrualEvent {
    Initialized {
        id: InterestAccrualId,
        facility_id: CreditFacilityId,
        idx: InterestAccrualIdx,
        started_at: DateTime<Utc>,
        facility_expires_at: DateTime<Utc>,
        terms: TermValues,
        audit_info: AuditInfo,
    },
    InterestIncurred {
        amount: UsdCents,
        incurred_at: DateTime<Utc>,
    },
    InterestAccrued {
        total: UsdCents,
        accrued_at: DateTime<Utc>,
    },
}

impl EntityEvent for InterestAccrualEvent {
    type EntityId = InterestAccrualId;
    fn event_table_name() -> &'static str {
        "interest_accrual_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct InterestAccrual {
    pub id: InterestAccrualId,
    pub facility_id: CreditFacilityId,
    pub idx: InterestAccrualIdx,
    pub started_at: DateTime<Utc>,
    pub facility_expires_at: DateTime<Utc>,
    pub terms: TermValues,
    pub(super) events: EntityEvents<InterestAccrualEvent>,
}

impl Entity for InterestAccrual {
    type Event = InterestAccrualEvent;
}

impl TryFrom<EntityEvents<InterestAccrualEvent>> for InterestAccrual {
    type Error = EntityError;

    fn try_from(events: EntityEvents<InterestAccrualEvent>) -> Result<Self, Self::Error> {
        let mut builder = InterestAccrualBuilder::default();
        for event in events.iter() {
            match event {
                InterestAccrualEvent::Initialized {
                    id,
                    facility_id,
                    idx,
                    started_at,
                    facility_expires_at,
                    terms,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .facility_id(*facility_id)
                        .idx(*idx)
                        .started_at(*started_at)
                        .facility_expires_at(*facility_expires_at)
                        .terms(*terms)
                }
                InterestAccrualEvent::InterestIncurred { .. } => (),
                InterestAccrualEvent::InterestAccrued { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

impl InterestAccrual {
    fn accrues_at(&self) -> DateTime<Utc> {
        self.terms
            .accrual_interval
            .period_from(self.started_at)
            .truncate(self.facility_expires_at)
            .expect("'started_at' should be before 'facility_expires_at'")
            .end
    }

    pub fn is_accrued(&self) -> bool {
        for event in self.events.iter() {
            match event {
                InterestAccrualEvent::InterestAccrued { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    fn total_incurred(&self) -> UsdCents {
        self.events
            .iter()
            .filter_map(|event| match event {
                InterestAccrualEvent::InterestIncurred { amount, .. } => Some(*amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
    }

    fn next_incurrence_period(&self) -> Result<InterestPeriod, InterestAccrualError> {
        let last_incurrence = self.events.iter().rev().find_map(|event| match event {
            InterestAccrualEvent::InterestIncurred { incurred_at, .. } => Some(*incurred_at),
            _ => None,
        });

        let incurrence_interval = self
            .terms
            .incurrence_interval
            .expect("'incurrence_interval' should exist");

        let untruncated_period = match last_incurrence {
            Some(last_end_date) => incurrence_interval.period_from(last_end_date).next(),
            None => incurrence_interval.period_from(self.started_at),
        };

        Ok(untruncated_period
            .truncate(self.accrues_at())
            .ok_or(InterestAccrualError::InterestPeriodStartDatePastAccrualDate)?)
    }

    fn initiate_incurrence(
        &mut self,
        outstanding: OutstandingForPeriod,
    ) -> Result<Option<()>, InterestAccrualError> {
        if self.is_accrued() {
            return Err(InterestAccrualError::AlreadyAccrued);
        }

        let OutstandingForPeriod {
            period: incurrence_period,
            amount: outstanding_amount,
        } = outstanding;
        if incurrence_period != self.next_incurrence_period()? {
            return Err(InterestAccrualError::NonCurrentIncurrencePeriod);
        }

        let secs_in_interest_period = incurrence_period.seconds();
        let interest_for_period = self
            .terms
            .annual_rate
            .interest_for_time_period_in_secs(outstanding_amount, secs_in_interest_period);

        self.events.push(InterestAccrualEvent::InterestIncurred {
            amount: interest_for_period,
            incurred_at: incurrence_period.end,
        });

        if incurrence_period
            .next()
            .truncate(self.accrues_at())
            .is_none()
        {
            self.events.push(InterestAccrualEvent::InterestAccrued {
                total: self.total_incurred(),
                accrued_at: incurrence_period.end,
            });
            return Ok(Some(()));
        } else {
            return Ok(None);
        }
    }
}

#[derive(Debug)]
pub struct NewInterestAccrual {
    pub(super) id: InterestAccrualId,
    pub(super) facility_id: CreditFacilityId,
    pub(super) idx: InterestAccrualIdx,
    pub(super) started_at: DateTime<Utc>,
    pub(super) facility_expires_at: DateTime<Utc>,
    pub(super) terms: TermValues,
    pub(super) audit_info: AuditInfo,
}

impl NewInterestAccrual {
    pub fn new(
        facility_id: CreditFacilityId,
        idx: InterestAccrualIdx,
        started_at: DateTime<Utc>,
        facility_expires_at: DateTime<Utc>,
        terms: TermValues,
        audit_info: AuditInfo,
    ) -> Self {
        Self {
            id: InterestAccrualId::new(),
            facility_id,
            idx,
            started_at,
            facility_expires_at,
            terms,
            audit_info,
        }
    }

    pub fn initial_events(self) -> EntityEvents<InterestAccrualEvent> {
        EntityEvents::init(
            self.id,
            [InterestAccrualEvent::Initialized {
                id: self.id,
                facility_id: self.facility_id,
                idx: self.idx,
                started_at: self.started_at,
                facility_expires_at: self.facility_expires_at,
                terms: self.terms,
                audit_info: self.audit_info,
            }],
        )
    }
}
