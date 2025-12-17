use chrono::{DateTime, Datelike, Days, Months, NaiveDate, Utc};
use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::error::*;
use tracing_macros::record_error_severity;

use crate::primitives::{ChartId, FiscalYearId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "FiscalYearId")]
pub enum FiscalYearEvent {
    Initialized {
        id: FiscalYearId,
        chart_id: ChartId,
        reference: String,
        opened_as_of: chrono::NaiveDate,
    },
    MonthClosed {
        month_closed_as_of: NaiveDate,
        month_closed_at: DateTime<Utc>,
    },
    YearClosed {
        closed_as_of: NaiveDate,
        closed_at: DateTime<Utc>,
    },
}

#[derive(EsEntity, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct FiscalYear {
    pub id: FiscalYearId,
    pub chart_id: ChartId,
    pub reference: String,
    pub opened_as_of: NaiveDate,
    #[builder(default)]
    pub closed_as_of: Option<NaiveDate>,
    events: EntityEvents<FiscalYearEvent>,
}

pub struct FiscalMonthClosure {
    pub closed_as_of: NaiveDate,
    pub closed_at: DateTime<Utc>,
}

impl FiscalYear {
    #[record_error_severity]
    #[instrument(name = "fiscal_year.close", skip(self, now))]
    pub(super) fn close(
        &mut self,
        now: DateTime<Utc>,
    ) -> Result<Idempotent<NaiveDate>, FiscalYearError> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            FiscalYearEvent::YearClosed { .. }
        );
        if !self.is_last_month_of_year_closed() {
            return Err(FiscalYearError::LastMonthNotClosed);
        }

        let closed_as_of = self.closes_as_of();
        self.events.push(FiscalYearEvent::YearClosed {
            closed_as_of,
            closed_at: now,
        });
        self.closed_as_of = Some(closed_as_of);
        Ok(Idempotent::Executed(closed_as_of))
    }

    pub fn closes_as_of(&self) -> NaiveDate {
        let year = self.opened_as_of.year();
        NaiveDate::from_ymd_opt(year, 12, 31).expect("Failed to compute december of fiscal year")
    }

    #[record_error_severity]
    #[instrument(name = "fiscal_year.close_next_sequential_month", skip(self, now))]
    pub(super) fn close_next_sequential_month(
        &mut self,
        now: DateTime<Utc>,
    ) -> Idempotent<NaiveDate> {
        if self.is_last_month_of_year_closed() {
            return Idempotent::Ignored;
        }
        let last_recorded_date = self.events.iter_all().rev().find_map(|event| match event {
            FiscalYearEvent::MonthClosed {
                month_closed_as_of, ..
            } => Some(*month_closed_as_of),
            _ => None,
        });
        let new_monthly_closing_date = match last_recorded_date {
            Some(last_effective) => {
                let last_day_of_previous_month = now
                    .date_naive()
                    .with_day(1)
                    .and_then(|d| d.pred_opt())
                    .expect("Failed to compute last day of previous month");
                if last_effective == last_day_of_previous_month {
                    return Idempotent::Ignored;
                }

                last_effective
                    .checked_add_months(Months::new(2))
                    .and_then(|d| d.with_day(1))
                    .and_then(|d| d.pred_opt())
                    .expect("Failed to compute new monthly closing date")
            }
            None => self
                .events
                .iter_all()
                .find_map(|event| match event {
                    FiscalYearEvent::Initialized { opened_as_of, .. } => Some(opened_as_of),
                    _ => None,
                })
                .expect("Entity was not initialized")
                .checked_add_months(Months::new(1))
                .and_then(|d| d.with_day(1))
                .and_then(|d| d.pred_opt())
                .expect("Failed to compute new monthly closing date"),
        };

        self.events.push(FiscalYearEvent::MonthClosed {
            month_closed_as_of: new_monthly_closing_date,
            month_closed_at: now,
        });
        Idempotent::Executed(new_monthly_closing_date)
    }

    pub fn month_closures(&self) -> Vec<FiscalMonthClosure> {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                FiscalYearEvent::MonthClosed {
                    month_closed_as_of,
                    month_closed_at,
                } => Some(FiscalMonthClosure {
                    closed_as_of: *month_closed_as_of,
                    closed_at: *month_closed_at,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn is_last_month_of_year_closed(&self) -> bool {
        let last_month_closes_as_of = self.closes_as_of();
        self.events
            .iter_all()
            .rev()
            .any(|event| matches!(
                event,
                FiscalYearEvent::MonthClosed { month_closed_as_of, .. } if *month_closed_as_of == last_month_closes_as_of
            ))
    }

    pub fn year(&self) -> String {
        self.opened_as_of.year().to_string()
    }

    pub fn next_month_to_close(&self) -> Option<NaiveDate> {
        if self.is_last_month_of_year_closed() {
            return None;
        }

        let last_recorded_date = self.events.iter_all().rev().find_map(|event| match event {
            FiscalYearEvent::MonthClosed {
                month_closed_as_of, ..
            } => Some(*month_closed_as_of),
            _ => None,
        });

        let next_month = match last_recorded_date {
            Some(last_effective) => last_effective
                .checked_add_months(Months::new(2))
                .and_then(|d| d.with_day(1))
                .and_then(|d| d.pred_opt())
                .expect("Failed to compute new monthly closing date"),
            None => self
                .events
                .iter_all()
                .find_map(|event| match event {
                    FiscalYearEvent::Initialized { opened_as_of, .. } => Some(opened_as_of),
                    _ => None,
                })
                .expect("Entity was not initialized")
                .checked_add_months(Months::new(1))
                .and_then(|d| d.with_day(1))
                .and_then(|d| d.pred_opt())
                .expect("Failed to compute new monthly closing date"),
        };

        Some(next_month)
    }

    #[record_error_severity]
    #[instrument(name = "fiscal_year.next", skip(self))]
    pub(super) fn next(&self, now: DateTime<Utc>) -> Result<NewFiscalYear, FiscalYearError> {
        let next_year_opened_as_of = self
            .closes_as_of()
            .checked_add_days(Days::new(1))
            .expect("Failed to compute start of next fiscal year");
        if next_year_opened_as_of.year() > now.date_naive().year() + 1 {
            return Err(FiscalYearError::FiscalYearWithInvalidOpenedAsOf(
                next_year_opened_as_of,
            ));
        }

        let next_fiscal_year = NewFiscalYear::builder()
            .id(FiscalYearId::new())
            .chart_id(self.chart_id)
            .opened_as_of(next_year_opened_as_of)
            .build()
            .expect("Could not build new fiscal year");

        Ok(next_fiscal_year)
    }
}

impl TryFromEvents<FiscalYearEvent> for FiscalYear {
    fn try_from_events(events: EntityEvents<FiscalYearEvent>) -> Result<Self, EsEntityError> {
        let mut builder = FiscalYearBuilder::default();

        for event in events.iter_all() {
            match event {
                FiscalYearEvent::Initialized {
                    id,
                    chart_id,
                    reference,
                    opened_as_of,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .chart_id(*chart_id)
                        .reference(reference.to_string())
                        .opened_as_of(*opened_as_of)
                }
                FiscalYearEvent::MonthClosed { .. } => {}
                FiscalYearEvent::YearClosed { closed_as_of, .. } => {
                    builder = builder.closed_as_of(Some(*closed_as_of));
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewFiscalYear {
    #[builder(setter(into))]
    pub id: FiscalYearId,
    #[builder(setter(into))]
    pub chart_id: ChartId,
    pub opened_as_of: NaiveDate,
}

impl NewFiscalYear {
    pub fn builder() -> NewFiscalYearBuilder {
        NewFiscalYearBuilder::default()
    }

    pub(super) fn reference(&self) -> String {
        self.opened_as_of.year().to_string()
    }
}

impl IntoEvents<FiscalYearEvent> for NewFiscalYear {
    fn into_events(self) -> EntityEvents<FiscalYearEvent> {
        EntityEvents::init(
            self.id,
            [FiscalYearEvent::Initialized {
                id: self.id,
                chart_id: self.chart_id,
                reference: self.reference(),
                opened_as_of: self.opened_as_of,
            }],
        )
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn fiscal_year_from(events: Vec<FiscalYearEvent>) -> FiscalYear {
        FiscalYear::try_from_events(EntityEvents::init(FiscalYearId::new(), events)).unwrap()
    }

    fn initial_events_with_opened_date(opened_as_of: NaiveDate) -> Vec<FiscalYearEvent> {
        vec![FiscalYearEvent::Initialized {
            id: FiscalYearId::new(),
            chart_id: ChartId::new(),
            reference: "2025".to_string(),
            opened_as_of,
        }]
    }

    #[test]
    fn close_next_sequential_month_first_time() {
        let period_start = "2024-01-01".parse::<NaiveDate>().unwrap();
        let expected_closed_date = "2024-01-31".parse::<NaiveDate>().unwrap();
        let mut fiscal_year = fiscal_year_from(initial_events_with_opened_date(period_start));

        let timestamp = Utc::now();
        let closed_date = fiscal_year.close_next_sequential_month(timestamp).unwrap();
        assert_eq!(closed_date, expected_closed_date);

        let closing_event_date = fiscal_year
            .events
            .iter_all()
            .rev()
            .find_map(|e| match e {
                FiscalYearEvent::MonthClosed {
                    month_closed_as_of, ..
                } => Some(*month_closed_as_of),
                _ => None,
            })
            .unwrap();
        assert_eq!(closing_event_date, expected_closed_date);
    }

    #[test]
    fn close_next_sequential_month_after_prior_closes() {
        let period_start = "2024-01-01".parse::<NaiveDate>().unwrap();
        let expected_second_closed_date = "2024-02-29".parse::<NaiveDate>().unwrap();
        let mut fiscal_year = fiscal_year_from(initial_events_with_opened_date(period_start));

        let _ = fiscal_year.close_next_sequential_month(Utc::now()).unwrap();

        let second_closing_ts = Utc::now();
        let second_closing_date = fiscal_year
            .close_next_sequential_month(second_closing_ts)
            .unwrap();
        assert_eq!(second_closing_date, expected_second_closed_date);
    }

    #[test]
    fn close_next_sequential_month_ignored_for_current_month() {
        let first_day_of_last_month = chrono::Utc::now()
            .date_naive()
            .checked_sub_months(chrono::Months::new(1))
            .and_then(|d| d.with_day(1))
            .unwrap();
        let mut fiscal_year =
            fiscal_year_from(initial_events_with_opened_date(first_day_of_last_month));
        let _ = fiscal_year.close_next_sequential_month(Utc::now()).unwrap();
        let second_closing_date = fiscal_year.close_next_sequential_month(Utc::now());
        assert!(second_closing_date.was_ignored());
    }

    #[test]
    fn close_fails_when_december_not_closed() {
        let period_start = "2024-01-01".parse::<NaiveDate>().unwrap();
        let mut fiscal_year = fiscal_year_from(initial_events_with_opened_date(period_start));

        let _ = fiscal_year.close_next_sequential_month(Utc::now()).unwrap();
        let result = fiscal_year.close(Utc::now());
        assert!(result.is_err());
        assert!(matches!(result, Err(FiscalYearError::LastMonthNotClosed)));
    }

    #[test]
    fn close_succeeds_when_december_closed() {
        let period_start = "2024-12-01".parse::<NaiveDate>().unwrap();
        let mut fiscal_year = fiscal_year_from(initial_events_with_opened_date(period_start));

        let _ = fiscal_year.close_next_sequential_month(Utc::now()).unwrap();
        let result = fiscal_year.close(Utc::now());
        assert!(result.is_ok());

        let db_op = result.unwrap();
        assert!(db_op.did_execute());

        let closed_as_of = db_op.unwrap();
        assert_eq!(closed_as_of, "2024-12-31".parse::<NaiveDate>().unwrap());
    }

    #[test]
    fn close_ignored_when_already_closed() {
        let period_start = "2024-12-01".parse::<NaiveDate>().unwrap();
        let mut fiscal_year = fiscal_year_from(initial_events_with_opened_date(period_start));

        let _ = fiscal_year.close_next_sequential_month(Utc::now()).unwrap();
        let _ = fiscal_year.close(Utc::now());

        let second_closing = fiscal_year.close(Utc::now());
        assert!(second_closing.is_ok());
        assert!(second_closing.unwrap().was_ignored());
    }

    #[test]
    fn next_fails_when_year_is_invalid() {
        let now = Utc::now().date_naive();
        let period_start = now.checked_add_months(Months::new(12)).unwrap();
        let fiscal_year = fiscal_year_from(initial_events_with_opened_date(period_start));
        let result = fiscal_year.next(Utc::now());

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(FiscalYearError::FiscalYearWithInvalidOpenedAsOf(_))
        ));
    }

    #[test]
    fn next_succeeds_when_year_is_valid() {
        let period_start = Utc::now().date_naive();
        let fiscal_year = fiscal_year_from(initial_events_with_opened_date(period_start));
        let result = fiscal_year.next(Utc::now());
        assert!(result.is_ok());
    }
}
