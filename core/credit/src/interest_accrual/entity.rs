use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{
    audit::AuditInfo,
    credit_facility::{CreditFacilityInterestAccrual, CreditFacilityReceivable},
    primitives::*,
    terms::{InterestPeriod, TermValues},
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "InterestAccrualId")]
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

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct InterestAccrual {
    pub id: InterestAccrualId,
    pub credit_facility_id: CreditFacilityId,
    pub idx: InterestAccrualIdx,
    pub started_at: DateTime<Utc>,
    pub facility_expires_at: DateTime<Utc>,
    pub terms: TermValues,
    pub(super) events: EntityEvents<InterestAccrualEvent>,
}

#[derive(Debug, Clone)]
pub(crate) struct InterestAccrualData {
    pub(crate) interest: UsdCents,
    pub(crate) tx_ref: String,
    pub(crate) tx_id: LedgerTxId,
    pub(crate) accrued_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub(crate) struct InterestIncurrenceData {
    pub(crate) interest: UsdCents,
    pub(crate) period: InterestPeriod,
    pub(crate) tx_ref: String,
    pub(crate) tx_id: LedgerTxId,
}

impl TryFromEvents<InterestAccrualEvent> for InterestAccrual {
    fn try_from_events(events: EntityEvents<InterestAccrualEvent>) -> Result<Self, EsEntityError> {
        let mut builder = InterestAccrualBuilder::default();
        for event in events.iter_all() {
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
                        .credit_facility_id(*facility_id)
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
            .iter_all()
            .any(|event| matches!(event, InterestAccrualEvent::InterestAccrued { .. }))
    }

    fn total_incurred(&self) -> UsdCents {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                InterestAccrualEvent::InterestIncurred { amount, .. } => Some(*amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
    }

    fn count_incurred(&self) -> usize {
        self.events
            .iter_all()
            .filter(|event| matches!(event, InterestAccrualEvent::InterestIncurred { .. }))
            .count()
    }

    fn last_incurrence_period(&self) -> Option<InterestPeriod> {
        let mut last_incurred_at = None;
        let mut second_to_last_incurred_at = None;
        for event in self.events.iter_all() {
            if let InterestAccrualEvent::InterestIncurred { incurred_at, .. } = event {
                second_to_last_incurred_at = last_incurred_at;
                last_incurred_at = Some(*incurred_at);
            }
        }
        last_incurred_at?;

        let interval = self.terms.incurrence_interval;
        match second_to_last_incurred_at {
            Some(incurred_at) => interval.period_from(incurred_at).next(),
            None => interval.period_from(self.started_at),
        }
        .truncate(self.accrues_at())
    }

    pub(crate) fn next_incurrence_period(&self) -> Option<InterestPeriod> {
        let last_incurrence = self.events.iter_all().rev().find_map(|event| match event {
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

    pub(crate) fn record_incurrence(
        &mut self,
        outstanding: CreditFacilityReceivable,
        audit_info: AuditInfo,
    ) -> InterestIncurrenceData {
        let incurrence_period = self
            .next_incurrence_period()
            .expect("Incurrence period should exist inside this function");

        let days_in_interest_period = incurrence_period.days();
        let interest_for_period = self
            .terms
            .annual_rate
            .interest_for_time_period(outstanding.total(), days_in_interest_period);

        let incurrence_tx_ref = format!(
            "{}-interest-incurrence-{}",
            self.id,
            self.count_incurred() + 1
        );
        let interest_incurrence = InterestIncurrenceData {
            interest: interest_for_period,
            period: incurrence_period,
            tx_ref: incurrence_tx_ref,
            tx_id: LedgerTxId::new(),
        };

        self.events.push(InterestAccrualEvent::InterestIncurred {
            tx_id: interest_incurrence.tx_id,
            tx_ref: interest_incurrence.tx_ref.to_string(),
            amount: interest_incurrence.interest,
            incurred_at: interest_incurrence.period.end,
            audit_info,
        });

        interest_incurrence
    }

    pub(crate) fn accrual_data(&self) -> Option<InterestAccrualData> {
        let last_incurrence_period = self.last_incurrence_period()?;

        match last_incurrence_period.next().truncate(self.accrues_at()) {
            Some(_) => None,
            None => {
                let accrual_tx_ref =
                    format!("{}-interest-accrual-{}", self.credit_facility_id, self.idx);
                let interest_accrual = InterestAccrualData {
                    interest: self.total_incurred(),
                    tx_ref: accrual_tx_ref,
                    tx_id: LedgerTxId::new(),
                    accrued_at: last_incurrence_period.end,
                };

                Some(interest_accrual)
            }
        }
    }

    pub(crate) fn record_accrual(
        &mut self,
        CreditFacilityInterestAccrual {
            interest,
            tx_ref,
            tx_id,
            accrued_at,
            ..
        }: CreditFacilityInterestAccrual,
        audit_info: AuditInfo,
    ) {
        self.events.push(InterestAccrualEvent::InterestAccrued {
            tx_id,
            tx_ref,
            total: interest,
            accrued_at,
            audit_info,
        });
    }
}

#[derive(Debug, Builder)]
pub struct NewInterestAccrual {
    #[builder(setter(into))]
    pub(in crate::credit_facility) id: InterestAccrualId,
    #[builder(setter(into))]
    pub(in crate::credit_facility) credit_facility_id: CreditFacilityId,
    pub(in crate::credit_facility) idx: InterestAccrualIdx,
    pub(in crate::credit_facility) started_at: DateTime<Utc>,
    pub(in crate::credit_facility) facility_expires_at: DateTime<Utc>,
    terms: TermValues,
    #[builder(setter(into))]
    audit_info: AuditInfo,
}

impl NewInterestAccrual {
    pub fn builder() -> NewInterestAccrualBuilder {
        NewInterestAccrualBuilder::default()
    }

    pub fn first_incurrence_period(&self) -> InterestPeriod {
        self.terms.incurrence_interval.period_from(self.started_at)
    }
}

impl IntoEvents<InterestAccrualEvent> for NewInterestAccrual {
    fn into_events(self) -> EntityEvents<InterestAccrualEvent> {
        EntityEvents::init(
            self.id,
            [InterestAccrualEvent::Initialized {
                id: self.id,
                facility_id: self.credit_facility_id,
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
    use chrono::{Datelike, TimeZone, Utc};
    use rust_decimal_macros::dec;

    use crate::{
        audit::AuditEntryId,
        terms::{Duration, InterestInterval, OneTimeFeeRatePct},
    };

    use super::*;

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(3))
            .accrual_interval(InterestInterval::EndOfMonth)
            .incurrence_interval(InterestInterval::EndOfDay)
            .one_time_fee_rate(OneTimeFeeRatePct::ZERO)
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
            sub: "sub".to_string(),
        }
    }

    fn accrual_from(events: Vec<InterestAccrualEvent>) -> InterestAccrual {
        InterestAccrual::try_from_events(EntityEvents::init(InterestAccrualId::new(), events))
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
    fn last_incurrence_period_at_start() {
        let accrual = accrual_from(initial_events());
        assert_eq!(accrual.last_incurrence_period(), None,);
    }

    #[test]
    fn last_incurrence_period_in_middle() {
        let mut events = initial_events();

        let first_incurrence_period = default_terms()
            .incurrence_interval
            .period_from(default_started_at());
        let first_incurrence_at = first_incurrence_period.end;
        events.push(InterestAccrualEvent::InterestIncurred {
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            incurred_at: first_incurrence_at,
            audit_info: dummy_audit_info(),
        });
        let accrual = accrual_from(events.clone());
        assert_eq!(
            accrual.last_incurrence_period().unwrap().start,
            accrual.started_at
        );

        let second_incurrence_period = first_incurrence_period.next();
        let second_incurrence_at = second_incurrence_period.end;
        events.push(InterestAccrualEvent::InterestIncurred {
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            incurred_at: second_incurrence_at,
            audit_info: dummy_audit_info(),
        });
        let accrual = accrual_from(events);
        assert_eq!(
            accrual.last_incurrence_period().unwrap().start,
            second_incurrence_period.start
        );
    }

    #[test]
    fn next_incurrence_period_at_start() {
        let accrual = accrual_from(initial_events());
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
        let accrual = accrual_from(events);

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
        let accrual = accrual_from(events);

        assert_eq!(accrual.next_incurrence_period(), None);
    }

    #[test]
    fn zero_amount_incurrence() {
        let mut accrual = accrual_from(initial_events());
        let InterestIncurrenceData {
            interest, period, ..
        } = accrual.record_incurrence(
            CreditFacilityReceivable {
                disbursed: UsdCents::ZERO,
                interest: UsdCents::ZERO,
            },
            dummy_audit_info(),
        );
        assert_eq!(interest, UsdCents::ZERO);
        let start = default_started_at();
        assert_eq!(period.start, start);
        let start = start.date_naive();
        let end_of_day = Utc
            .with_ymd_and_hms(start.year(), start.month(), start.day(), 23, 59, 59)
            .unwrap();
        assert_eq!(period.end, end_of_day);

        assert!(accrual.accrual_data().is_none());
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

    #[test]
    fn accrual_is_zero_for_zero_outstanding() {
        let mut accrual = accrual_from(initial_events());

        let start = default_started_at();
        let start_day = start.day();
        let end = end_of_month(start);
        let end_day = end.day();
        let mut expected_end_of_day = Utc
            .with_ymd_and_hms(start.year(), start.month(), start.day(), 23, 59, 59)
            .unwrap();
        let mut confirmed_incurrence: Option<InterestAccrualData> = None;
        for _ in start_day..(end_day + 1) {
            assert!(confirmed_incurrence.is_none());

            let InterestIncurrenceData {
                interest, period, ..
            } = accrual.record_incurrence(
                CreditFacilityReceivable {
                    disbursed: UsdCents::ZERO,
                    interest: UsdCents::ZERO,
                },
                dummy_audit_info(),
            );
            assert_eq!(interest, UsdCents::ZERO);
            assert_eq!(period.end, expected_end_of_day);

            confirmed_incurrence = accrual.accrual_data();
            expected_end_of_day += chrono::Duration::days(1);
        }

        let expected_accrual_sum = UsdCents::ZERO;
        match confirmed_incurrence {
            Some(InterestAccrualData { interest, .. }) => {
                assert_eq!(interest, expected_accrual_sum);
            }
            _ => panic!("Expected accrual to be returned"),
        }
    }

    #[test]
    fn accrual_is_sum_of_all_interest() {
        let disbursed_outstanding = UsdCents::from(1_000_000_00);
        let expected_daily_interest = default_terms()
            .annual_rate
            .interest_for_time_period(disbursed_outstanding, 1);

        let mut accrual = accrual_from(initial_events());

        let start = default_started_at();
        let start_day = start.day();
        let end = end_of_month(start);
        let end_day = end.day();
        let mut expected_end_of_day = Utc
            .with_ymd_and_hms(start.year(), start.month(), start.day(), 23, 59, 59)
            .unwrap();
        let mut confirmed_incurrence: Option<InterestAccrualData> = None;
        for _ in start_day..(end_day + 1) {
            assert!(confirmed_incurrence.is_none());

            let InterestIncurrenceData {
                interest, period, ..
            } = accrual.record_incurrence(
                CreditFacilityReceivable {
                    disbursed: disbursed_outstanding,
                    interest: UsdCents::ZERO,
                },
                dummy_audit_info(),
            );
            assert_eq!(interest, expected_daily_interest);
            assert_eq!(period.end, expected_end_of_day);

            confirmed_incurrence = accrual.accrual_data();
            expected_end_of_day += chrono::Duration::days(1);
        }

        let expected_accrual_sum = expected_daily_interest * (end_day + 1 - start_day).into();
        match confirmed_incurrence {
            Some(InterestAccrualData { interest, .. }) => {
                assert_eq!(interest, expected_accrual_sum);
            }
            _ => panic!("Expected accrual to be returned"),
        }
    }
}
