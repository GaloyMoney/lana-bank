use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    credit_facility::{
        CreditFacilityAccountIds, CreditFacilityInterestAccrual, CreditFacilityInterestIncurrence,
        CreditFacilityReceivable,
    },
    entity::{Entity, EntityError, EntityEvent, EntityEvents},
    primitives::*,
    terms::{InterestPeriod, TermValues},
};

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
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
        incurred_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    InterestAccrued {
        tx_id: LedgerTxId,
        tx_ref: String,
        total: UsdCents,
        accrued_at: DateTime<Utc>,
        audit_info: AuditInfo,
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
        self.events
            .iter()
            .any(|event| matches!(event, InterestAccrualEvent::InterestAccrued { .. }))
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

    fn count_incurred(&self) -> usize {
        self.events
            .iter()
            .filter(|event| matches!(event, InterestAccrualEvent::InterestIncurred { .. }))
            .count()
    }

    pub fn next_incurrence_period(&self) -> Option<InterestPeriod> {
        let last_incurrence = self.events.iter().rev().find_map(|event| match event {
            InterestAccrualEvent::InterestIncurred { incurred_at, .. } => Some(*incurred_at),
            _ => None,
        });

        let incurrence_interval = self.terms.incurrence_interval;

        let untruncated_period = match last_incurrence {
            Some(last_end_date) => incurrence_interval.period_from(last_end_date).next(),
            None => incurrence_interval.period_from(self.started_at),
        };

        untruncated_period.truncate(self.accrues_at())
    }

    pub fn initiate_incurrence(
        &mut self,
        outstanding: CreditFacilityReceivable,
        credit_facility_account_ids: CreditFacilityAccountIds,
    ) -> CreditFacilityInterestIncurrence {
        let incurrence_period = self
            .next_incurrence_period()
            .expect("Incurrence period should exist inside this function");

        let secs_in_interest_period = incurrence_period.seconds();
        let interest_for_period = self
            .terms
            .annual_rate
            .interest_for_time_period_in_secs(outstanding.total(), secs_in_interest_period);

        let incurrence_tx_ref = format!(
            "{}-interest-incurrence-{}",
            self.id,
            self.count_incurred() + 1
        );
        CreditFacilityInterestIncurrence {
            interest: interest_for_period,
            period: incurrence_period,
            tx_ref: incurrence_tx_ref,
            tx_id: LedgerTxId::new(),
            credit_facility_account_ids,
        }
    }

    pub fn confirm_incurrence(
        &mut self,
        CreditFacilityInterestIncurrence {
            interest,
            tx_ref,
            tx_id,
            period,
            ..
        }: CreditFacilityInterestIncurrence,
        credit_facility_account_ids: CreditFacilityAccountIds,
        audit_info: AuditInfo,
    ) -> Option<CreditFacilityInterestAccrual> {
        self.events.push(InterestAccrualEvent::InterestIncurred {
            tx_id,
            tx_ref,
            amount: interest,
            incurred_at: period.end,
            audit_info,
        });

        match period.next().truncate(self.accrues_at()) {
            Some(_) => None,
            None => {
                let accrual_tx_ref = format!("{}-interest-accrual-{}", self.facility_id, self.idx);
                let interest_accrual = CreditFacilityInterestAccrual {
                    interest: self.total_incurred(),
                    tx_ref: accrual_tx_ref,
                    tx_id: LedgerTxId::new(),
                    credit_facility_account_ids,
                };

                Some(interest_accrual)
            }
        }
    }

    pub fn confirm_accrual(
        &mut self,
        CreditFacilityInterestAccrual {
            interest,
            tx_ref,
            tx_id,
            ..
        }: CreditFacilityInterestAccrual,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) {
        self.events.push(InterestAccrualEvent::InterestAccrued {
            tx_id,
            tx_ref,
            total: interest,
            accrued_at: executed_at,
            audit_info,
        });
    }
}

#[derive(Debug, Builder)]
pub struct NewInterestAccrual {
    #[builder(setter(into))]
    pub(super) id: InterestAccrualId,
    #[builder(setter(into))]
    pub(super) facility_id: CreditFacilityId,
    pub(super) idx: InterestAccrualIdx,
    pub(super) started_at: DateTime<Utc>,
    pub(super) facility_expires_at: DateTime<Utc>,
    pub(super) terms: TermValues,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
}

impl NewInterestAccrual {
    pub fn builder() -> NewInterestAccrualBuilder {
        NewInterestAccrualBuilder::default()
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

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use crate::terms::{Duration, InterestInterval};

    use super::*;

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(3))
            .accrual_interval(InterestInterval::EndOfMonth)
            .incurrence_interval(InterestInterval::EndOfDay)
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term")
    }

    fn default_started_at() -> DateTime<Utc> {
        "2024-01-15T12:00:00Z".parse::<DateTime<Utc>>().unwrap()
    }

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: Subject::from(UserId::new()),
        }
    }

    fn accrual_from(events: &Vec<InterestAccrualEvent>) -> InterestAccrual {
        InterestAccrual::try_from(EntityEvents::init(InterestAccrualId::new(), events.clone()))
            .unwrap()
    }

    fn initial_events() -> Vec<InterestAccrualEvent> {
        let terms = default_terms();
        let started_at = default_started_at();
        vec![InterestAccrualEvent::Initialized {
            id: InterestAccrualId::new(),
            facility_id: CreditFacilityId::new(),
            idx: InterestAccrualIdx::FIRST,
            started_at,
            facility_expires_at: terms.duration.expiration_date(started_at),
            terms,
            audit_info: dummy_audit_info(),
        }]
    }

    #[test]
    fn next_incurrence_period_at_start() {
        let accrual = accrual_from(&initial_events());
        assert_eq!(
            accrual.next_incurrence_period().unwrap().start,
            accrual.started_at
        );
    }

    #[test]
    fn next_incurrence_period_in_middle() {
        let mut events = initial_events();

        let first_incurrence_period = default_terms()
            .incurrence_interval
            .period_from(default_started_at());
        let first_incurrence_at = first_incurrence_period.end;
        events.extend([InterestAccrualEvent::InterestIncurred {
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            incurred_at: first_incurrence_at,
            audit_info: dummy_audit_info(),
        }]);
        let accrual = accrual_from(&events);

        assert_eq!(
            accrual.next_incurrence_period().unwrap(),
            first_incurrence_period.next()
        );
    }

    #[test]
    fn next_incurrence_period_at_end() {
        let mut events = initial_events();

        let facility_expires_at = default_terms()
            .duration
            .expiration_date(default_started_at());
        let final_incurrence_period = default_terms()
            .accrual_interval
            .period_from(default_started_at())
            .truncate(facility_expires_at)
            .unwrap();
        let final_incurrence_at = final_incurrence_period.end;

        events.extend([InterestAccrualEvent::InterestIncurred {
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            incurred_at: final_incurrence_at,
            audit_info: dummy_audit_info(),
        }]);
        let accrual = accrual_from(&events);

        assert_eq!(accrual.next_incurrence_period(), None);
    }
}
