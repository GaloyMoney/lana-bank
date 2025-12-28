use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{
    ledger::InterestAccrualCycleLedgerAccountIds,
    obligation::NewObligation,
    primitives::*,
    terms::{InterestPeriod, TermValues},
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "InterestAccrualCycleId")]
pub enum InterestAccrualCycleEvent {
    Initialized {
        id: InterestAccrualCycleId,
        facility_id: CreditFacilityId,
        idx: InterestAccrualCycleIdx,
        period: InterestPeriod,
        facility_maturity_date: EffectiveDate,
        account_ids: InterestAccrualCycleLedgerAccountIds,
        terms: TermValues,
    },
    InterestAccrued {
        ledger_tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
        accrued_at: DateTime<Utc>,
    },
    InterestAccrualsPosted {
        ledger_tx_id: LedgerTxId,
        tx_ref: String,
        obligation_id: Option<ObligationId>,
        total: UsdCents,
        effective: chrono::NaiveDate,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct InterestAccrualCycle {
    pub id: InterestAccrualCycleId,
    pub credit_facility_id: CreditFacilityId,
    pub account_ids: InterestAccrualCycleLedgerAccountIds,
    pub idx: InterestAccrualCycleIdx,
    pub facility_maturity_date: EffectiveDate,
    pub terms: TermValues,
    pub period: InterestPeriod,
    events: EntityEvents<InterestAccrualCycleEvent>,
}

#[derive(Debug, Clone)]
pub(crate) struct InterestAccrualCycleData {
    pub(crate) interest: UsdCents,
    pub(crate) tx_ref: String,
    pub(crate) tx_id: LedgerTxId,
    pub(crate) effective: chrono::NaiveDate,
}

#[derive(Debug, Clone)]
pub(crate) struct NewInterestAccrualCycleData {
    pub(crate) id: InterestAccrualCycleId,
    pub(crate) first_accrual_end_date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub(crate) struct InterestAccrualData {
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
                    account_ids,
                    idx,
                    period,
                    facility_maturity_date,
                    terms,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .credit_facility_id(*facility_id)
                        .account_ids(*account_ids)
                        .idx(*idx)
                        .period(*period)
                        .facility_maturity_date(*facility_maturity_date)
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
    fn accrual_cycle_ends_at(&self) -> EffectiveDate {
        self.terms
            .accrual_cycle_interval
            .period_from(self.period.start)
            .truncate(self.facility_maturity_date.start_of_day())
            .expect("'period.start' should be before 'facility_maturity_date'")
            .end
            .into()
    }

    fn total_accrued(&self) -> UsdCents {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                InterestAccrualCycleEvent::InterestAccrued { amount, .. } => Some(*amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
    }

    fn last_accrual_period(&self) -> Option<InterestPeriod> {
        let mut last_accrued_at = None;
        let mut second_to_last_accrued_at = None;
        for event in self.events.iter_all() {
            if let InterestAccrualCycleEvent::InterestAccrued { accrued_at, .. } = event {
                second_to_last_accrued_at = last_accrued_at;
                last_accrued_at = Some(*accrued_at);
            }
        }
        last_accrued_at?;

        let interval = self.terms.accrual_interval;
        match second_to_last_accrued_at {
            Some(accrued_at) => interval.period_from(accrued_at).next(),
            None => interval.period_from(self.period.start),
        }
        .truncate(self.accrual_cycle_ends_at().end_of_day())
    }

    pub(crate) fn is_completed(&self) -> bool {
        self.events.iter_all().rev().any(|event| {
            matches!(
                event,
                InterestAccrualCycleEvent::InterestAccrualsPosted { .. }
            )
        })
    }

    pub fn count_accrued(&self) -> usize {
        self.events
            .iter_all()
            .filter(|event| matches!(event, InterestAccrualCycleEvent::InterestAccrued { .. }))
            .count()
    }

    pub(crate) fn next_accrual_period(&self) -> Option<InterestPeriod> {
        let last_accrual = self.events.iter_all().rev().find_map(|event| match event {
            InterestAccrualCycleEvent::InterestAccrued { accrued_at, .. } => Some(*accrued_at),
            _ => None,
        });

        let accrual_interval = self.terms.accrual_interval;

        let untruncated_period = match last_accrual {
            Some(last_end_date) => accrual_interval.period_from(last_end_date).next(),
            None => accrual_interval.period_from(self.period.start),
        };

        untruncated_period.truncate(self.accrual_cycle_ends_at().end_of_day())
    }

    pub(crate) fn record_accrual(&mut self, amount: UsdCents) -> InterestAccrualData {
        let accrual_period = self
            .next_accrual_period()
            .expect("Accrual period should exist inside this function");

        let days_in_interest_period = accrual_period.days();
        let interest_for_period = self
            .terms
            .annual_rate
            .interest_for_time_period(amount, days_in_interest_period);

        let accrual_tx_ref = format!("{}-interest-accrual-{}", self.id, self.count_accrued() + 1);
        let interest_accrual = InterestAccrualData {
            interest: interest_for_period,
            period: accrual_period,
            tx_ref: accrual_tx_ref,
            tx_id: LedgerTxId::new(),
        };

        self.events
            .push(InterestAccrualCycleEvent::InterestAccrued {
                ledger_tx_id: interest_accrual.tx_id,
                tx_ref: interest_accrual.tx_ref.to_string(),
                amount: interest_accrual.interest,
                accrued_at: interest_accrual.period.end,
            });

        interest_accrual
    }

    pub(crate) fn accrual_cycle_data(&self) -> Option<InterestAccrualCycleData> {
        let last_accrual_period = self.last_accrual_period()?;

        match last_accrual_period
            .next()
            .truncate(self.accrual_cycle_ends_at().end_of_day())
        {
            Some(_) => None,
            None => {
                let accrual_cycle_tx_ref = format!(
                    "{}-interest-accrual-cycle-{}",
                    self.credit_facility_id, self.idx
                );
                let interest_accrual_cycle = InterestAccrualCycleData {
                    interest: self.total_accrued(),
                    tx_ref: accrual_cycle_tx_ref,
                    tx_id: LedgerTxId::new(),
                    effective: last_accrual_period.end.date_naive(),
                };

                Some(interest_accrual_cycle)
            }
        }
    }

    pub(crate) fn record_accrual_cycle(
        &mut self,
        InterestAccrualCycleData {
            interest,
            tx_ref,
            tx_id,
            effective,
            ..
        }: InterestAccrualCycleData,
    ) -> Idempotent<Option<NewObligation>> {
        idempotency_guard!(
            self.events.iter_all(),
            InterestAccrualCycleEvent::InterestAccrualsPosted { .. }
        );

        if interest.is_zero() {
            self.events
                .push(InterestAccrualCycleEvent::InterestAccrualsPosted {
                    ledger_tx_id: tx_id,
                    tx_ref: tx_ref.to_string(),
                    obligation_id: None,
                    total: interest,
                    effective,
                });

            return Idempotent::Executed(None);
        }

        let due_date = self.accrual_cycle_ends_at();
        let overdue_date = self
            .terms
            .obligation_overdue_duration_from_due
            .map(|d| d.end_date(due_date));
        let liquidation_date = self
            .terms
            .obligation_liquidation_duration_from_due
            .map(|d| d.end_date(due_date));
        let new_obligation = NewObligation::builder()
            .id(ObligationId::new())
            .credit_facility_id(self.credit_facility_id)
            .obligation_type(ObligationType::Interest)
            .reference(tx_ref.to_string())
            .amount(interest)
            .tx_id(tx_id)
            .receivable_account_ids(self.account_ids.into())
            .defaulted_account_id(self.account_ids.defaulted_account_id)
            .due_date(due_date)
            .overdue_date(overdue_date)
            .liquidation_date(liquidation_date)
            .effective(effective)
            .build()
            .expect("could not build new interest accrual cycle obligation");

        self.events
            .push(InterestAccrualCycleEvent::InterestAccrualsPosted {
                ledger_tx_id: tx_id,
                tx_ref: tx_ref.to_string(),
                obligation_id: Some(new_obligation.id),
                total: interest,
                effective,
            });

        Idempotent::Executed(Some(new_obligation))
    }
}

#[derive(Debug, Builder)]
pub struct NewInterestAccrualCycle {
    #[builder(setter(into))]
    pub id: InterestAccrualCycleId,
    #[builder(setter(into))]
    pub credit_facility_id: CreditFacilityId,
    pub account_ids: InterestAccrualCycleLedgerAccountIds,
    pub idx: InterestAccrualCycleIdx,
    pub period: InterestPeriod,
    pub facility_maturity_date: EffectiveDate,
    terms: TermValues,
}

impl NewInterestAccrualCycle {
    pub fn builder() -> NewInterestAccrualCycleBuilder {
        NewInterestAccrualCycleBuilder::default()
    }

    pub fn first_accrual_cycle_period(&self) -> InterestPeriod {
        self.terms.accrual_interval.period_from(self.period.start)
    }
}

impl IntoEvents<InterestAccrualCycleEvent> for NewInterestAccrualCycle {
    fn into_events(self) -> EntityEvents<InterestAccrualCycleEvent> {
        EntityEvents::init(
            self.id,
            [InterestAccrualCycleEvent::Initialized {
                id: self.id,
                facility_id: self.credit_facility_id,
                account_ids: self.account_ids,
                idx: self.idx,
                period: self.period,
                facility_maturity_date: self.facility_maturity_date,
                terms: self.terms,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::inconsistent_digit_grouping)]

    use chrono::{Datelike, TimeZone, Utc};
    use rust_decimal_macros::dec;

    use crate::{
        ledger::CreditFacilityLedgerAccountIds,
        terms::{
            DisbursalPolicy, FacilityDuration, InterestInterval, ObligationDuration,
            OneTimeFeeRatePct,
        },
    };

    use super::*;

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(FacilityDuration::Months(3))
            .interest_due_duration_from_accrual(ObligationDuration::Days(0))
            .obligation_overdue_duration_from_due(None)
            .obligation_liquidation_duration_from_due(None)
            .accrual_cycle_interval(InterestInterval::EndOfMonth)
            .accrual_interval(InterestInterval::EndOfDay)
            .one_time_fee_rate(OneTimeFeeRatePct::ZERO)
            .disbursal_policy(DisbursalPolicy::SingleDisbursal)
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term")
    }

    fn default_started_at() -> DateTime<Utc> {
        "2024-01-15T12:00:00Z".parse::<DateTime<Utc>>().unwrap()
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

    fn default_period() -> InterestPeriod {
        InterestInterval::EndOfDay.period_from(default_started_at())
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
            account_ids: CreditFacilityLedgerAccountIds::new().into(),
            idx: InterestAccrualCycleIdx::FIRST,
            period: default_period(),
            facility_maturity_date: terms.maturity_date(started_at),
            terms,
        }]
    }

    #[test]
    fn last_accrual_period_at_start() {
        let accrual = accrual_from(initial_events());
        assert_eq!(accrual.last_accrual_period(), None,);
    }

    #[test]
    fn last_accrual_period_in_middle() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_accrual_at,
        });
        let accrual = accrual_from(events.clone());
        assert_eq!(
            accrual.last_accrual_period().unwrap().start,
            accrual.period.start
        );

        let second_accrual_period = first_accrual_cycle_period.next();
        let second_accrual_at = second_accrual_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: second_accrual_at,
        });
        let accrual = accrual_from(events);
        assert_eq!(
            accrual.last_accrual_period().unwrap().start,
            second_accrual_period.start
        );
    }

    #[test]
    fn count_accrued_period_at_start() {
        let accrual = accrual_from(initial_events());
        assert_eq!(accrual.count_accrued(), 0);
    }

    #[test]
    fn count_multiple_accrued() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_accrual_at,
        });
        let accrual = accrual_from(events.clone());
        assert_eq!(accrual.count_accrued(), 1);

        let second_accrual_period = first_accrual_cycle_period.next();
        let second_accrual_at = second_accrual_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: second_accrual_at,
        });
        let accrual = accrual_from(events);
        assert_eq!(accrual.count_accrued(), 2);
    }

    #[test]
    fn next_accrual_period_at_start() {
        let accrual = accrual_from(initial_events());
        assert_eq!(
            accrual.next_accrual_period().unwrap().start,
            accrual.period.start
        );
    }

    #[test]
    fn next_accrual_period_in_middle() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.extend([InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_accrual_at,
        }]);
        let accrual = accrual_from(events);

        assert_eq!(
            accrual.next_accrual_period().unwrap(),
            first_accrual_cycle_period.next()
        );
    }

    #[test]
    fn next_accrual_period_at_end() {
        let mut events = initial_events();

        let facility_maturity_date = default_terms().maturity_date(default_started_at());
        let final_accrual_period = default_terms()
            .accrual_cycle_interval
            .period_from(default_started_at())
            .truncate(facility_maturity_date.start_of_day())
            .unwrap();
        let final_accrual_at = final_accrual_period.end;

        events.extend([InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: final_accrual_at,
        }]);
        let accrual = accrual_from(events);

        assert_eq!(accrual.next_accrual_period(), None);
    }

    #[test]
    fn total_accrued() {
        let mut events = initial_events();

        let first_accrual_cycle_period = default_terms()
            .accrual_interval
            .period_from(default_started_at());
        let first_accrual_at = first_accrual_cycle_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: first_accrual_at,
        });

        let second_accrual_period = first_accrual_cycle_period.next();
        let second_accrual_at = second_accrual_period.end;
        events.push(InterestAccrualCycleEvent::InterestAccrued {
            ledger_tx_id: LedgerTxId::new(),
            tx_ref: "".to_string(),
            amount: UsdCents::ONE,
            accrued_at: second_accrual_at,
        });

        let accrual = accrual_from(events);
        assert_eq!(accrual.total_accrued(), UsdCents::from(2));
    }

    #[test]
    fn zero_amount_accrual() {
        let mut accrual = accrual_from(initial_events());
        let InterestAccrualData {
            interest, period, ..
        } = accrual.record_accrual(UsdCents::ZERO);
        assert_eq!(interest, UsdCents::ZERO);
        let start = default_started_at();
        assert_eq!(period.start, start);
        let start = start.date_naive();
        let end_of_day = Utc
            .with_ymd_and_hms(start.year(), start.month(), start.day(), 23, 59, 59)
            .unwrap();
        assert_eq!(period.end, end_of_day);

        assert!(accrual.accrual_cycle_data().is_none());
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
        let mut accrual_cycle_data: Option<InterestAccrualCycleData> = None;
        for _ in start_day..(end_day + 1) {
            assert!(accrual_cycle_data.is_none());

            let InterestAccrualData {
                interest, period, ..
            } = accrual.record_accrual(UsdCents::ZERO);
            assert_eq!(interest, UsdCents::ZERO);
            assert_eq!(period.end, expected_end_of_day);

            accrual_cycle_data = accrual.accrual_cycle_data();
            expected_end_of_day += chrono::Duration::days(1);
        }

        let expected_accrual_sum = UsdCents::ZERO;
        match accrual_cycle_data {
            Some(InterestAccrualCycleData { interest, .. }) => {
                assert_eq!(interest, expected_accrual_sum);
            }
            _ => panic!("Expected accrual to be returned"),
        }
    }

    #[test]
    fn record_accrual_returns_correct_period() {
        let mut accrual = accrual_from(initial_events());

        let start = default_started_at();
        let end = end_of_month(start);
        let start_day = start.day();
        let end_day = end.day();

        let mut expected_end_of_day = Utc
            .with_ymd_and_hms(start.year(), start.month(), start.day(), 23, 59, 59)
            .unwrap();
        for _ in start_day..(end_day + 1) {
            let InterestAccrualData { period, .. } = accrual.record_accrual(UsdCents::ONE);
            assert_eq!(period.end, expected_end_of_day);

            expected_end_of_day += chrono::Duration::days(1);
        }
    }

    #[test]
    fn accrual_cycle_data_exists_at_end_of_cycle() {
        let mut accrual = accrual_from(initial_events());

        let start = default_started_at();
        let end = end_of_month(start);
        let start_day = start.day();
        let end_day = end.day();

        let mut accrual_cycle_data: Option<InterestAccrualCycleData> = None;
        for _ in start_day..(end_day + 1) {
            assert!(accrual_cycle_data.is_none());

            accrual.record_accrual(UsdCents::ONE);

            accrual_cycle_data = accrual.accrual_cycle_data();
        }
        assert!(accrual_cycle_data.is_some());
    }

    #[test]
    fn accrual_is_sum_of_all_interest() {
        let disbursed_outstanding_amount = UsdCents::from(1_000_000_00);
        let expected_daily_interest = default_terms()
            .annual_rate
            .interest_for_time_period(disbursed_outstanding_amount, 1);

        let mut accrual = accrual_from(initial_events());

        let start = default_started_at();
        let end = end_of_month(start);
        let start_day = start.day();
        let end_day = end.day();

        for _ in start_day..(end_day + 1) {
            let InterestAccrualData { interest, .. } =
                accrual.record_accrual(disbursed_outstanding_amount);
            assert_eq!(interest, expected_daily_interest);
        }

        let expected_accrual_sum = expected_daily_interest * (end_day + 1 - start_day).into();
        let InterestAccrualCycleData { interest, .. } = accrual.accrual_cycle_data().unwrap();
        assert_eq!(interest, expected_accrual_sum);
    }
}
