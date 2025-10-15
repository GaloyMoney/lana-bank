use chrono::{DateTime, Datelike as _, Days, Duration, Months, NaiveDate, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::{AccountSetId as LedgerAccountSetId, TransactionId as LedgerTransactionId};
use es_entity::*;

use crate::{AccountingPeriodId, primitives::ChartId};

use super::error::AccountingPeriodError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Year {
    Calendar,
    Fiscal { first: NaiveDate },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Month {
    Calendar,
    OnDay(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Frequency {
    Year(Year),
    Month(Month),
}

impl Frequency {
    pub const fn is_monthly(&self) -> bool {
        matches!(self, Frequency::Month(..))
    }

    /// Returns end date of a period with this frequency and for given
    /// `period_start`. Returns `None` if `period_start` does not
    /// match with the frequency.
    pub fn period_end(&self, period_start: &NaiveDate) -> Option<NaiveDate> {
        match self {
            Frequency::Year(Year::Calendar) => {
                if period_start.ordinal() == 1 {
                    Some(
                        period_start
                            .with_year(period_start.year() + 1)
                            .expect("January 1st is always valid")
                            .checked_sub_days(Days::new(1))
                            .expect("always in valid date range"),
                    )
                } else {
                    None
                }
            }
            Frequency::Year(Year::Fiscal { first }) => {
                if period_start.ordinal() > 1
                    && period_start.day() == first.day()
                    && period_start.month() == first.month()
                {
                    Some(
                        first
                            .checked_sub_days(Days::new(1))
                            .expect("always in valid date range")
                            .with_year(period_start.year() + 1)
                            .expect("cannot hit 2/29"),
                    )
                } else {
                    None
                }
            }
            Frequency::Month(Month::Calendar) => {
                if period_start.day() == 1 {
                    Some(
                        period_start
                            .with_day(period_start.num_days_in_month().into())
                            .expect("always in valid date range"),
                    )
                } else {
                    None
                }
            }
            Frequency::Month(Month::OnDay(d)) => {
                let d: u32 = (*d).into();
                if period_start.day() == d {
                    Some(
                        period_start
                            .with_day(d)
                            .expect("always in valid date range")
                            .checked_add_months(Months::new(1))
                            .expect("always in valid date range (add month truncates)")
                            .checked_sub_days(Days::new(1))
                            .expect("always in valid date range"),
                    )
                } else {
                    None
                }
            }
        }
    }
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AccountingPeriodId")]
pub enum AccountingPeriodEvent {
    Initialized {
        id: AccountingPeriodId,
        chart_id: ChartId,
        tracking_account_set: LedgerAccountSetId,
        frequency: Frequency,
        period_start: NaiveDate,
        period_end: NaiveDate,
        grace_period: Duration,
    },
    Closed {
        closing_transaction: Option<LedgerTransactionId>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct AccountingPeriod {
    pub(super) id: AccountingPeriodId,
    chart_id: ChartId,
    tracking_account_set: LedgerAccountSetId,
    pub(super) frequency: Frequency,
    period_start: NaiveDate,
    period_end: NaiveDate,
    grace_period: Duration,

    events: EntityEvents<AccountingPeriodEvent>,
}

impl AccountingPeriod {
    /// Unconditionally closes this Accounting Period. Returns a
    /// blueprint for the next Accounting Period or
    /// `Idempotent::Ignored` if the period has already been closed.
    ///
    /// This method does not verify any temporal conditions. Call
    /// `checked_close` if temporal conditions have to be verified.
    pub fn close(
        &mut self,
        closing_transaction: Option<LedgerTransactionId>,
    ) -> Result<Idempotent<NewAccountingPeriod>, AccountingPeriodError> {
        idempotency_guard!(self.events.iter_all(), AccountingPeriodEvent::Closed { .. });

        let new_period_start = self.period_end.checked_add_days(Days::new(1)).unwrap();
        let new_period_end = self
            .frequency
            .period_end(&new_period_start)
            .ok_or(AccountingPeriodError::CannotCalculatePeriodEnd)?;

        let new_period = NewAccountingPeriod {
            id: AccountingPeriodId::new(),
            chart_id: self.chart_id,
            frequency: self.frequency.clone(),
            tracking_account_set: self.tracking_account_set,
            period_start: new_period_start,
            period_end: new_period_end,
            grace_period: self.grace_period.clone(),
        };

        self.events.push(AccountingPeriodEvent::Closed {
            closing_transaction,
        });

        Ok(Idempotent::Executed(new_period))
    }

    /// Closes this Accounting Period if all temporal conditions are
    /// met, otherwise returns an error describing the unfulfilled
    /// condition. Returns a blueprint for the next Accounting Period
    /// or `Idempotent::Ignored` if the period has already been
    /// closed.
    ///
    /// To close unconditionally call `close`.
    pub fn checked_close(
        &mut self,
        closing_transaction: Option<LedgerTransactionId>,
        closing_date: NaiveDate,
    ) -> Result<Idempotent<NewAccountingPeriod>, AccountingPeriodError> {
        self.check_can_close(closing_date)?;
        self.close(closing_transaction)
    }

    /// Verifies that `closing_date` falls into allowable time range,
    /// i. e. between the end of this period and the end of grace
    /// period. Returns error otherwise.
    fn check_can_close(&self, closing_date: NaiveDate) -> Result<(), AccountingPeriodError> {
        if closing_date < self.period_start {
            Err(AccountingPeriodError::ClosingDateBeforePeriodStart {
                closing_date,
                period_start: self.period_start,
            })
        } else if closing_date < self.period_end {
            Err(AccountingPeriodError::ClosingDateBeforePeriodEnd {
                closing_date,
                period_end: self.period_end,
            })
        } else if closing_date > self.period_end + self.grace_period {
            Err(AccountingPeriodError::ClosingDateAfterGracePeriod {
                closing_date,
                grace_period_end: self.period_end + self.grace_period,
            })
        } else {
            Ok(())
        }
    }
}

impl TryFromEvents<AccountingPeriodEvent> for AccountingPeriod {
    fn try_from_events(events: EntityEvents<AccountingPeriodEvent>) -> Result<Self, EsEntityError> {
        let mut builder = AccountingPeriodBuilder::default();

        for event in events.iter_all() {
            match event {
                AccountingPeriodEvent::Initialized {
                    id,
                    chart_id,
                    tracking_account_set,
                    frequency,
                    period_start,
                    period_end,
                    grace_period,
                } => {
                    builder = builder
                        .id(*id)
                        .chart_id(*chart_id)
                        .tracking_account_set(*tracking_account_set)
                        .frequency(frequency.clone())
                        .period_start(*period_start)
                        .period_end(*period_end)
                        .grace_period(*grace_period);
                }
                AccountingPeriodEvent::Closed { .. } => {}
            }
        }

        builder.events(events).build()
    }
}

pub struct NewAccountingPeriod {
    pub id: AccountingPeriodId,
    pub chart_id: ChartId,
    pub tracking_account_set: LedgerAccountSetId,
    pub frequency: Frequency,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub grace_period: Duration,
}

impl IntoEvents<AccountingPeriodEvent> for NewAccountingPeriod {
    fn into_events(self) -> EntityEvents<AccountingPeriodEvent> {
        let mut events = vec![AccountingPeriodEvent::Initialized {
            id: self.id,
            chart_id: self.chart_id,
            tracking_account_set: self.tracking_account_set,
            frequency: self.frequency,
            period_start: self.period_start,
            period_end: self.period_end,
            grace_period: self.grace_period,
        }];

        EntityEvents::init(self.id, events)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{Frequency, Month, Year};

    fn dt(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn test(freq: &Frequency, start: &str, expected: &str) {
        assert_eq!(freq.period_end(&dt(start)), Some(dt(expected)));
    }

    fn fail(freq: &Frequency, start: &str) {
        assert!(freq.period_end(&dt(start)).is_none());
    }

    #[test]
    fn frequency_calendar_month() {
        let freq = Frequency::Month(Month::Calendar);

        test(&freq, "2025-05-01", "2025-05-31");
        test(&freq, "2025-04-01", "2025-04-30");
        test(&freq, "2025-03-01", "2025-03-31");
        test(&freq, "2025-12-01", "2025-12-31");
        test(&freq, "2025-01-01", "2025-01-31");

        fail(&freq, "2025-02-02");
        fail(&freq, "2025-01-31");
        fail(&freq, "2025-09-22");
    }

    #[test]
    fn frequency_month_onday() {
        let freq = Frequency::Month(Month::OnDay(12));

        test(&freq, "2025-05-12", "2025-06-11");
        test(&freq, "2025-04-12", "2025-05-11");
        test(&freq, "2025-03-12", "2025-04-11");
        test(&freq, "2025-12-12", "2026-01-11");
        test(&freq, "2025-01-12", "2025-02-11");

        fail(&freq, "2025-01-01");
        fail(&freq, "2025-01-13");
        fail(&freq, "2025-01-11");
        fail(&freq, "2025-01-31");

        // These are equivalent to "last day of month starts new period"
        let freq = Frequency::Month(Month::OnDay(31));
        test(&freq, "2025-01-31", "2025-02-27");
        test(&freq, "2025-03-31", "2025-04-29");
    }

    #[test]
    fn frequency_calendar_year() {
        let freq = Frequency::Year(Year::Calendar);

        test(&freq, "2025-01-01", "2025-12-31");
        test(&freq, "2023-01-01", "2023-12-31");

        fail(&freq, "2025-01-02");
        fail(&freq, "2025-01-13");
        fail(&freq, "2025-01-31");
        fail(&freq, "2025-12-31");
    }

    #[test]
    fn frequency_fiscal_calendar() {
        fn freq(first: NaiveDate) -> Frequency {
            Frequency::Year(Year::Fiscal { first })
        }

        test(&freq(dt("2025-05-01")), "2025-05-01", "2026-04-30");
        test(&freq(dt("2022-04-01")), "2025-04-01", "2026-03-31");
        test(&freq(dt("2023-03-01")), "2025-03-01", "2026-02-28");
        test(&freq(dt("2024-02-29")), "2024-02-29", "2025-02-28");
        test(&freq(dt("2020-12-01")), "2025-12-01", "2026-11-30");

        fail(&freq(dt("2021-01-01")), "2025-01-01");
        fail(&freq(dt("2024-01-02")), "2025-01-01");
        fail(&freq(dt("2025-12-30")), "2025-12-31");
    }
}
