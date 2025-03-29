use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::{
    credit_facility::CreditFacilityReceivable,
    primitives::*,
    terms::{InterestPeriod, TermValues},
    CreditFacilityInterestAccrualsPosting,
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "InterestAccrualCycleId")]
pub enum InterestAccrualCycleEvent {
    Initialized {
        id: InterestAccrualCycleId,
        facility_id: CreditFacilityId,
        idx: InterestAccrualCycleIdx,
        started_at: DateTime<Utc>,
        facility_matures_at: DateTime<Utc>,
        terms: TermValues,
        audit_info: AuditInfo,
    },
    InterestAccrued {
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
        accrued_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    InterestAccrualsPosted {
        tx_id: LedgerTxId,
        tx_ref: String,
        total: UsdCents,
        posted_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct InterestAccrualCycle {
    pub id: InterestAccrualCycleId,
    pub credit_facility_id: CreditFacilityId,
    pub idx: InterestAccrualCycleIdx,
    pub started_at: DateTime<Utc>,
    pub facility_matures_at: DateTime<Utc>,
    pub terms: TermValues,
    pub(super) events: EntityEvents<InterestAccrualCycleEvent>,
}

#[derive(Debug, Clone)]
pub(crate) struct InterestAccrualsPostingData {
    pub(crate) interest: UsdCents,
    pub(crate) tx_ref: String,
    pub(crate) tx_id: LedgerTxId,
    pub(crate) posted_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub(crate) struct InterestIncurrenceData {
    pub(crate) interest: UsdCents,
    pub(crate) period: InterestPeriod,
    pub(crate) tx_ref: String,
    pub(crate) tx_id: LedgerTxId,
}

impl TryFromEvents<InterestAccrualCycleEvent> for InterestAccrualCycle {
    fn try_from_events(
        events: EntityEvents<InterestAccrualCycleEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = InterestAccrualCycleBuilder::default();
        for event in events.iter_all() {
            match event {
                InterestAccrualCycleEvent::Initialized {
                    id,
                    facility_id,
                    idx,
                    started_at,
                    facility_matures_at,
                    terms,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .credit_facility_id(*facility_id)
                        .idx(*idx)
                        .started_at(*started_at)
                        .facility_matures_at(*facility_matures_at)
                        .terms(*terms)
                }
                InterestAccrualCycleEvent::InterestAccrued { .. } => (),
                InterestAccrualCycleEvent::InterestAccrualsPosted { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

impl InterestAccrualCycle {
    fn accruals_post_at(&self) -> DateTime<Utc> {
        self.terms
            .accrual_interval
            .period_from(self.started_at)
            .truncate(self.facility_matures_at)
            .expect("'started_at' should be before 'facility_matures_at'")
            .end
    }

    pub fn is_accruals_posted(&self) -> bool {
        self.events.iter_all().any(|event| {
            matches!(
                event,
                InterestAccrualCycleEvent::InterestAccrualsPosted { .. }
            )
        })
    }

    fn total_incurred(&self) -> UsdCents {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                InterestAccrualCycleEvent::InterestAccrued { amount, .. } => Some(*amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
    }

    fn count_incurred(&self) -> usize {
        self.events
            .iter_all()
            .filter(|event| matches!(event, InterestAccrualCycleEvent::InterestAccrued { .. }))
            .count()
    }

    fn last_incurrence_period(&self) -> Option<InterestPeriod> {
        let mut last_accrued_at = None;
        let mut second_to_last_accrued_at = None;
        for event in self.events.iter_all() {
            if let InterestAccrualCycleEvent::InterestAccrued { accrued_at, .. } = event {
                second_to_last_accrued_at = last_accrued_at;
                last_accrued_at = Some(*accrued_at);
            }
        }
        last_accrued_at?;

        let interval = self.terms.incurrence_interval;
        match second_to_last_accrued_at {
            Some(accrued_at) => interval.period_from(accrued_at).next(),
            None => interval.period_from(self.started_at),
        }
        .truncate(self.accruals_post_at())
    }

    pub(crate) fn next_incurrence_period(&self) -> Option<InterestPeriod> {
        let last_incurrence = self.events.iter_all().rev().find_map(|event| match event {
            InterestAccrualCycleEvent::InterestAccrued { accrued_at, .. } => Some(*accrued_at),
            _ => None,
        });

        let incurrence_interval = self.terms.incurrence_interval;

        let untruncated_period = match last_incurrence {
            Some(last_end_date) => incurrence_interval.period_from(last_end_date).next(),
            None => incurrence_interval.period_from(self.started_at),
        };

        untruncated_period.truncate(self.accruals_post_at())
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

        self.events
            .push(InterestAccrualCycleEvent::InterestAccrued {
                tx_id: interest_incurrence.tx_id,
                tx_ref: interest_incurrence.tx_ref.to_string(),
                amount: interest_incurrence.interest,
                accrued_at: interest_incurrence.period.end,
                audit_info,
            });

        interest_incurrence
    }

    pub(crate) fn accruals_posting_data(&self) -> Option<InterestAccrualsPostingData> {
        let last_incurrence_period = self.last_incurrence_period()?;

        match last_incurrence_period
            .next()
            .truncate(self.accruals_post_at())
        {
            Some(_) => None,
            None => {
                let accruals_posting_tx_ref = format!(
                    "{}-interest-accruals-posting-{}",
                    self.credit_facility_id, self.idx
                );
                let interest_accruals_posting = InterestAccrualsPostingData {
                    interest: self.total_incurred(),
                    tx_ref: accruals_posting_tx_ref,
                    tx_id: LedgerTxId::new(),
                    posted_at: last_incurrence_period.end,
                };

                Some(interest_accruals_posting)
            }
        }
    }

    pub(crate) fn accumulate_unposted_accruals(
        &mut self,
        CreditFacilityInterestAccrualsPosting {
            interest,
            tx_ref,
            tx_id,
            posted_at,
            ..
        }: CreditFacilityInterestAccrualsPosting,
        audit_info: AuditInfo,
    ) {
        self.events
            .push(InterestAccrualCycleEvent::InterestAccrualsPosted {
                tx_id,
                tx_ref,
                total: interest,
                posted_at,
                audit_info,
            });
    }
}

#[derive(Debug, Builder)]
pub struct NewInterestAccrualCycle {
    #[builder(setter(into))]
    pub id: InterestAccrualCycleId,
    #[builder(setter(into))]
    pub credit_facility_id: CreditFacilityId,
    pub idx: InterestAccrualCycleIdx,
    pub started_at: DateTime<Utc>,
    pub facility_matures_at: DateTime<Utc>,
    terms: TermValues,
    #[builder(setter(into))]
    audit_info: AuditInfo,
}

impl NewInterestAccrualCycle {
    pub fn builder() -> NewInterestAccrualCycleBuilder {
        NewInterestAccrualCycleBuilder::default()
    }

    pub fn first_incurrence_period(&self) -> InterestPeriod {
        self.terms.incurrence_interval.period_from(self.started_at)
    }
}

impl IntoEvents<InterestAccrualCycleEvent> for NewInterestAccrualCycle {
    fn into_events(self) -> EntityEvents<InterestAccrualCycleEvent> {
        EntityEvents::init(
            self.id,
            [InterestAccrualCycleEvent::Initialized {
                id: self.id,
                facility_id: self.credit_facility_id,
                idx: self.idx,
                started_at: self.started_at,
                facility_matures_at: self.facility_matures_at,
                terms: self.terms,
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use audit::AuditEntryId;
    use chrono::{Datelike, TimeZone, Utc};
    use rust_decimal_macros::dec;

    use crate::terms::{Duration, InterestDuration, InterestInterval, OneTimeFeeRatePct};

    use super::*;

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(3))
            .interest_due_duration(InterestDuration::Days(0))
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

    fn accrual_from(events: Vec<InterestAccrualCycleEvent>) -> InterestAccrualCycle {
        InterestAccrualCycle::try_from_events(EntityEvents::init(
            InterestAccrualCycleId::new(),
            events,
        ))
        .unwrap()
    }

    fn initial_events() -> Vec<InterestAccrualCycleEvent> {
        let terms = default_terms();
        let started_at = default_started_at();
        vec![InterestAccrualCycleEvent::Initialized {
            id: InterestAccrualCycleId::new(),
            facility_id: CreditFacilityId::new(),
            idx: InterestAccrualCycleIdx::FIRST,
            started_at,
            facility_matures_at: terms.duration.maturity_date(started_at),
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
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_incurrence_at,
            audit_info: dummy_audit_info(),
        });
        let accrual = accrual_from(events.clone());
        assert_eq!(
            accrual.last_incurrence_period().unwrap().start,
            accrual.started_at
        );

        let second_incurrence_period = first_incurrence_period.next();
        let second_incurrence_at = second_incurrence_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: second_incurrence_at,
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
        events.extend([InterestAccrualCycleEvent::InterestAccrued {
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_incurrence_at,
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

        let facility_matures_at = default_terms().duration.maturity_date(default_started_at());
        let final_incurrence_period = default_terms()
            .accrual_interval
            .period_from(default_started_at())
            .truncate(facility_matures_at)
            .unwrap();
        let final_incurrence_at = final_incurrence_period.end;

        events.extend([InterestAccrualCycleEvent::InterestAccrued {
            tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: final_incurrence_at,
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

        assert!(accrual.accruals_posting_data().is_none());
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
        let mut confirmed_incurrence: Option<InterestAccrualsPostingData> = None;
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

            confirmed_incurrence = accrual.accruals_posting_data();
            expected_end_of_day += chrono::Duration::days(1);
        }

        let expected_accrual_sum = UsdCents::ZERO;
        match confirmed_incurrence {
            Some(InterestAccrualsPostingData { interest, .. }) => {
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
        let mut confirmed_incurrence: Option<InterestAccrualsPostingData> = None;
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

            confirmed_incurrence = accrual.accruals_posting_data();
            expected_end_of_day += chrono::Duration::days(1);
        }

        let expected_accrual_sum = expected_daily_interest * (end_day + 1 - start_day).into();
        match confirmed_incurrence {
            Some(InterestAccrualsPostingData { interest, .. }) => {
                assert_eq!(interest, expected_accrual_sum);
            }
            _ => panic!("Expected accrual to be returned"),
        }
    }
}
