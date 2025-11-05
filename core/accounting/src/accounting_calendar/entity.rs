use chrono::{DateTime, Datelike, Months, NaiveDate, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

use super::error::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AccountingCalendarId")]
pub enum AccountingCalendarEvent {
    Initialized {
        id: AccountingCalendarId,
        opened_as_of: NaiveDate,
        opened_at: DateTime<Utc>,
    },
    MonthlyClosed {
        closed_as_of: NaiveDate,
        closed_at: DateTime<Utc>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct AccountingCalendar {
    pub id: AccountingCalendarId,

    events: EntityEvents<AccountingCalendarEvent>,
}

impl AccountingCalendar {
    pub fn monthly_closing(&self) -> Option<PeriodClosing> {
        self.events.iter_all().rev().find_map(|event| match event {
            AccountingCalendarEvent::MonthlyClosed {
                closed_as_of,
                closed_at,
            } => Some(PeriodClosing::new(*closed_as_of, *closed_at)),
            _ => None,
        })
    }

    pub fn close_last_monthly_period(
        &mut self,
        now: DateTime<Utc>,
    ) -> Result<Idempotent<NaiveDate>, AccountingCalendarError> {
        let last_recorded_date = self.events.iter_all().rev().find_map(|event| match event {
            AccountingCalendarEvent::MonthlyClosed { closed_as_of, .. } => Some(*closed_as_of),
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
                    return Ok(Idempotent::Ignored);
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
                    AccountingCalendarEvent::Initialized { opened_as_of, .. } => {
                        Some(*opened_as_of)
                    }
                    _ => None,
                })
                .ok_or(AccountingCalendarError::AccountPeriodStartNotFound)?
                .checked_add_months(Months::new(1))
                .and_then(|d| d.with_day(1))
                .and_then(|d| d.pred_opt())
                .expect("Failed to compute new monthly closing date"),
        };

        self.events.push(AccountingCalendarEvent::MonthlyClosed {
            closed_as_of: new_monthly_closing_date,
            closed_at: now,
        });

        Ok(Idempotent::Executed(new_monthly_closing_date))
    }
}

impl TryFromEvents<AccountingCalendarEvent> for AccountingCalendar {
    fn try_from_events(
        events: EntityEvents<AccountingCalendarEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = AccountingCalendarBuilder::default();

        for event in events.iter_all() {
            match event {
                AccountingCalendarEvent::Initialized { id, .. } => builder = builder.id(*id),
                AccountingCalendarEvent::MonthlyClosed { .. } => (),
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewAccountingCalendar {
    #[builder(setter(into))]
    pub(super) id: AccountingCalendarId,
    pub(super) opened_as_of: NaiveDate,
    pub(super) opened_at: DateTime<Utc>,
}

impl NewAccountingCalendar {
    pub fn builder() -> NewAccountingCalendarBuilder {
        NewAccountingCalendarBuilder::default()
    }
}

impl IntoEvents<AccountingCalendarEvent> for NewAccountingCalendar {
    fn into_events(self) -> EntityEvents<AccountingCalendarEvent> {
        EntityEvents::init(
            self.id,
            [AccountingCalendarEvent::Initialized {
                id: self.id,
                opened_as_of: self.opened_as_of,
                opened_at: self.opened_at,
            }],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodClosing {
    pub closed_as_of: chrono::NaiveDate,
    pub closed_at: DateTime<Utc>,
}

impl PeriodClosing {
    fn new(effective: NaiveDate, recorded_at: DateTime<Utc>) -> Self {
        Self {
            closed_as_of: effective,
            closed_at: recorded_at,
        }
    }
}
