use chrono::{DateTime, Datelike, Months, NaiveDate, Utc};
use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;

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

    events: EntityEvents<FiscalYearEvent>,
}

impl FiscalYear {
    #[instrument(name = "fiscal_year.close_last_month", skip(self, now))]
    pub(super) fn close_last_month(&mut self, now: DateTime<Utc>) -> Idempotent<NaiveDate> {
        let last_recorded_date = self.events.iter_all().rev().find_map(|event| match event {
            FiscalYearEvent::MonthClosed { closed_as_of, .. } => Some(*closed_as_of),
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
            closed_as_of: new_monthly_closing_date,
            closed_at: now,
        });
        Idempotent::Executed(new_monthly_closing_date)
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
        format!("{}:AC{}", self.chart_id, self.opened_as_of.year())
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
            reference: "AC2025".to_string(),
            opened_as_of,
        }]
    }

    #[test]
    fn close_last_month_first_time() {
        let period_start = "2024-01-01".parse::<NaiveDate>().unwrap();
        let expected_closed_date = "2024-01-31".parse::<NaiveDate>().unwrap();
        let mut fiscal_year = fiscal_year_from(initial_events_with_opened_date(period_start));

        let timestamp = Utc::now();
        let closed_date = fiscal_year.close_last_month(timestamp).unwrap();
        assert_eq!(closed_date, expected_closed_date);

        let closing_event_date = fiscal_year
            .events
            .iter_all()
            .rev()
            .find_map(|e| match e {
                FiscalYearEvent::MonthClosed { closed_as_of, .. } => Some(*closed_as_of),
                _ => None,
            })
            .unwrap();
        assert_eq!(closing_event_date, expected_closed_date);
    }

    #[test]
    fn close_last_month_after_prior_closes() {
        let period_start = "2024-01-01".parse::<NaiveDate>().unwrap();
        let expected_second_closed_date = "2024-02-29".parse::<NaiveDate>().unwrap();
        let mut fiscal_year = fiscal_year_from(initial_events_with_opened_date(period_start));

        let _ = fiscal_year.close_last_month(Utc::now()).unwrap();

        let second_closing_ts = Utc::now();
        let second_closing_date = fiscal_year.close_last_month(second_closing_ts).unwrap();
        assert_eq!(second_closing_date, expected_second_closed_date);
    }

    #[test]
    fn close_last_month_ignored_for_current_month() {
        let first_day_of_last_month = chrono::Utc::now()
            .date_naive()
            .checked_sub_months(chrono::Months::new(1))
            .and_then(|d| d.with_day(1))
            .unwrap();
        let mut fiscal_year =
            fiscal_year_from(initial_events_with_opened_date(first_day_of_last_month));
        let _ = fiscal_year.close_last_month(Utc::now()).unwrap();
        let second_closing_date = fiscal_year.close_last_month(Utc::now());
        assert!(second_closing_date.was_ignored());
    }
}
