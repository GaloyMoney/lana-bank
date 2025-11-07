use chrono::{Datelike as _, Days, Duration, Months, NaiveDate};
use serde::{Deserialize, Serialize};

/// Recurring time interval (i. e. a portion of time between two
/// dates) with a _grace period_ after the period's end.
///
/// Grace period is typically used as a time buffer for some external,
/// time-sensitive action. It is purely informative and does not
/// contribute to the periodicity, i. e. the next period starts right
/// after the end of the previous period, regardless of grace period:
///
/// ```
/// S = start of period, E = end of period, G = end of grace period
///
/// S—————————————————E······G
///                   S—————————————————E······G
///                                     S—————————————————E······G
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Period {
    #[serde(flatten)]
    frequency: Frequency,
    period_start: NaiveDate,
    period_end: NaiveDate,
    grace_period_days: u8,
}

impl Period {
    /// Constructs new `Period` with monthly frequency and with
    /// periods starting on `period_start`.
    pub fn monthly(period_start: NaiveDate, grace_period_days: u8) -> Option<Self> {
        let frequency = Frequency::monthly(period_start.day())?;
        let period_end = frequency.period_end(&period_start);

        Some(Self {
            frequency,
            period_start,
            period_end,
            grace_period_days,
        })
    }

    /// Constructs new `Period` with monthly frequency, with periods
    /// starting on `day` and which is open around `date`
    /// (i. e. period starts before `date` and ends after `date`).
    pub fn monthly_around_date(day: u32, date: NaiveDate, grace_period_days: u8) -> Option<Self> {
        let period_start = date.with_day(day)?;

        let period_start = if period_start <= date {
            period_start
        } else {
            period_start
                .checked_sub_months(Months::new(1))
                .expect("always in valid range")
        };

        Period::monthly(period_start, grace_period_days)
    }

    /// Constructs new `Period` with annual frequency and with periods
    /// starting on `period_start`
    pub fn annually(period_start: NaiveDate, grace_period_days: u8) -> Option<Self> {
        let frequency = Frequency::annually(period_start.day(), period_start.month())?;
        let period_end = frequency.period_end(&period_start);

        Some(Self {
            frequency,
            period_start,
            period_end,
            grace_period_days,
        })
    }

    /// Constructs new `Period` with annual frequency, with periods
    /// starting on `day` and `month` and which is open around `date`
    /// (i. e. period starts before `date` and ends after `date`).
    pub fn annually_around_date(
        day: u32,
        month: u32,
        date: NaiveDate,
        grace_period_days: u8,
    ) -> Option<Self> {
        let period_start = date.with_day(day)?.with_month(month)?;

        let period_start = if period_start <= date {
            period_start
        } else {
            period_start.with_year(period_start.year() - 1)?
        };

        Period::annually(period_start, grace_period_days)
    }

    pub const fn is_monthly(&self) -> bool {
        self.frequency.is_monthly()
    }

    pub const fn is_annual(&self) -> bool {
        self.frequency.is_annual()
    }

    /// Returns new period immediately following this one.
    pub fn next(&self) -> Self {
        let new_period_start = self
            .period_end
            .checked_add_days(Days::new(1))
            .expect("always in correct date range");
        let new_period_end = self.frequency.period_end(&new_period_start);

        Self {
            period_start: new_period_start,
            period_end: new_period_end,
            frequency: self.frequency.clone(),
            grace_period_days: self.grace_period_days,
        }
    }

    pub fn is_within_grace_period(&self, date: NaiveDate) -> bool {
        date >= self.grace_period_start() && date <= self.grace_period_end()
    }

    pub const fn grace_period_start(&self) -> NaiveDate {
        self.period_end
    }

    pub fn grace_period_end(&self) -> NaiveDate {
        self.period_end + Duration::days(self.grace_period_days.into())
    }

    pub const fn period_end(&self) -> NaiveDate {
        self.period_end
    }

    pub const fn period_start(&self) -> NaiveDate {
        self.period_start
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
enum Frequency {
    Year { day: u32, month: u32 },
    Month { day: u32 },
}

impl Frequency {
    pub const fn monthly(day_in_month: u32) -> Option<Self> {
        if day_in_month > 0 && day_in_month < 28 {
            Some(Self::Month { day: day_in_month })
        } else {
            None
        }
    }

    pub const fn annually(day: u32, month: u32) -> Option<Self> {
        if day > 0 && day < 28 && month > 0 && month < 13 {
            Some(Self::Year { day, month })
        } else {
            None
        }
    }

    pub const fn is_monthly(&self) -> bool {
        matches!(self, Frequency::Month { .. })
    }

    pub const fn is_annual(&self) -> bool {
        matches!(self, Frequency::Year { .. })
    }

    /// Returns end date of a period with this frequency and for given
    /// `period_start`
    pub fn period_end(&self, period_start: &NaiveDate) -> NaiveDate {
        match self {
            Frequency::Year { day, month } => {
                NaiveDate::from_ymd_opt(period_start.year() + 1, *month, *day)
                    .expect("valid date")
                    .checked_sub_days(Days::new(1))
                    .expect("always in valid date range")
            }
            Frequency::Month { day } => period_start
                .with_day(*day)
                .expect("valid date")
                .checked_add_months(Months::new(1))
                .expect("always in valid date range (avoiding date >27)")
                .checked_sub_days(Days::new(1))
                .expect("always in valid date range"),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{Frequency, Period};

    fn dt(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn test(freq: &Frequency, start: &str, expected: &str) {
        assert_eq!(freq.period_end(&dt(start)), dt(expected));
    }

    #[test]
    fn frequency_calendar_month() {
        let freq = Frequency::monthly(1).unwrap();

        test(&freq, "2025-05-01", "2025-05-31");
        test(&freq, "2025-04-01", "2025-04-30");
        test(&freq, "2025-03-01", "2025-03-31");
        test(&freq, "2025-12-01", "2025-12-31");
        test(&freq, "2025-01-01", "2025-01-31");
    }

    #[test]
    fn frequency_month_onday() {
        let freq = Frequency::monthly(12).unwrap();

        test(&freq, "2025-05-12", "2025-06-11");
        test(&freq, "2025-04-12", "2025-05-11");
        test(&freq, "2025-03-12", "2025-04-11");
        test(&freq, "2025-12-12", "2026-01-11");
        test(&freq, "2025-01-12", "2025-02-11");

        fn freq2(day: u32) -> Option<Frequency> {
            Frequency::monthly(day)
        }

        assert!(freq2(0).is_none());
        assert!(freq2(28).is_none());
    }

    #[test]
    fn frequency_calendar_year() {
        let freq = Frequency::annually(1, 1).unwrap();

        test(&freq, "2025-01-01", "2025-12-31");
        test(&freq, "2023-01-01", "2023-12-31");
    }

    #[test]
    fn frequency_fiscal_calendar() {
        fn freq(day: u32, month: u32) -> Frequency {
            Frequency::annually(day, month).unwrap()
        }

        test(&freq(1, 5), "2025-05-01", "2026-04-30");
        test(&freq(1, 4), "2025-04-01", "2026-03-31");
        test(&freq(1, 3), "2025-03-01", "2026-02-28");
        test(&freq(1, 12), "2025-12-01", "2026-11-30");

        fn freq2(day: u32, month: u32) -> Option<Frequency> {
            Frequency::annually(day, month)
        }

        assert!(freq2(0, 4).is_none());
        assert!(freq2(4, 0).is_none());
        assert!(freq2(28, 10).is_none());
        assert!(freq2(10, 13).is_none());
    }

    #[test]
    fn around_dates() {
        let today = dt("2025-10-10");

        let test_month = |day: u32| {
            let x = Period::monthly_around_date(day, today, 5).unwrap();
            assert!(x.period_start() <= today);
            assert!(x.period_end() > today);
        };

        let test_year = |day: u32, month: u32| {
            let x = Period::annually_around_date(day, month, today, 5).unwrap();
            assert!(x.period_start() <= today);
            assert!(x.period_end() > today);
        };

        test_month(5);
        test_month(10);
        test_month(15);

        test_year(5, 10);
        test_year(10, 10);
        test_year(15, 10);
        test_year(10, 9);
        test_year(10, 11);
    }
}
