use chrono::{DateTime, Datelike as _, Days, Duration, Months, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    frequency: Frequency,
    period_start: NaiveDate,
    period_end: NaiveDate,
    grace_period_duration: Duration,
}

impl Period {
    pub const fn is_monthly(&self) -> bool {
        self.frequency.is_monthly()
    }

    /// Generates a new period immediately following this one. Returns
    /// `None` if next period cannot be calculated due to a date range
    /// mismatch.
    pub fn next(&self) -> Option<Self> {
        let new_period_start = self
            .period_end
            .checked_add_days(Days::new(1))
            .expect("always in correct date range");
        let new_period_end = self.frequency.period_end(&new_period_start)?;

        Some(Self {
            period_start: new_period_start,
            period_end: new_period_end,
            frequency: self.frequency.clone(),
            grace_period_duration: self.grace_period_duration.clone(),
        })
    }

    pub fn is_within_grace_period(&self, date: NaiveDate) -> bool {
        date >= self.grace_period_start() && date <= self.grace_period_end()
    }

    pub const fn grace_period_start(&self) -> NaiveDate {
        self.period_end
    }

    pub fn grace_period_end(&self) -> NaiveDate {
        self.period_end + self.grace_period_duration
    }
}

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
