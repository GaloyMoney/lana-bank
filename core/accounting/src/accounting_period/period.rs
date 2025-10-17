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
pub struct Period {
    frequency: Frequency,
    period_start: NaiveDate,
    period_end: NaiveDate,
    grace_period_duration: Duration,
}

impl Period {
    /// Constructs new `Period` with monthly frequency and with
    /// periods starting on the first day of a month and ending on the
    /// last day of that month. Returns `None` if `period_start` is
    /// not on the first day of a month.
    pub fn monthly_by_calendar(
        period_start: NaiveDate,
        grace_period_duration: Duration,
    ) -> Option<Self> {
        if period_start.day() == 1 {
            let frequency = Frequency::monthly_by_calendar();
            let period_end = frequency.period_end(&period_start);

            Some(Self {
                frequency,
                period_start,
                period_end,
                grace_period_duration,
            })
        } else {
            None
        }
    }

    /// Constructs new `Period` with monthly frequency and with
    /// periods starting on `day_in_month` every month and ending the
    /// next month one day before `day_in_month`. Returns `None` if
    /// `period_start` is not on `day_in_month`, if `day_in_month`
    /// equals to 1 (use [[monthly_by_calendar]] in such case) or if
    /// `day_in_month` is 28 or greater.
    pub fn monthly_by_day_in_month(
        day_in_month: u8,
        period_start: NaiveDate,
        grace_period_duration: Duration,
    ) -> Option<Self> {
        if period_start.day() == u32::from(day_in_month) {
            let frequency = Frequency::monthly_by_day_in_month(day_in_month)?;
            let period_end = frequency.period_end(&period_start);

            Some(Self {
                frequency,
                period_start,
                period_end,
                grace_period_duration,
            })
        } else {
            None
        }
    }

    /// Constructs new `Period` with annual frequency and with periods
    /// starting on the first day of a year and ending on the last day
    /// of that year. Returns `None` if `period_start` is not the
    /// first day of a year.
    pub fn annually_by_calendar(
        period_start: NaiveDate,
        grace_period_duration: Duration,
    ) -> Option<Self> {
        if period_start.ordinal() == 1 {
            let frequency = Frequency::annually_by_calendar();
            let period_end = frequency.period_end(&period_start);

            Some(Self {
                frequency,
                period_start,
                period_end,
                grace_period_duration,
            })
        } else {
            None
        }
    }

    /// Constructs new `Period` with annual frequency and with periods
    /// starting on `day` and `month` each year and ending the next
    /// year, one day before `day` and `month`. Returns `None` if
    /// `period_start` is on a first day of a year (use
    /// [[annually_by_calendar]] in such case), if `period_start` is
    /// not on `day` and `month`, if `day` is 28 or greated or if
    /// `month` is 13 or greater.
    pub fn annually_by_date(
        day: u8,
        month: u8,
        period_start: NaiveDate,
        grace_period_duration: Duration,
    ) -> Option<Self> {
        if period_start.ordinal() > 1
            && period_start.day() == u32::from(day)
            && period_start.month() == u32::from(month)
        {
            let frequency = Frequency::annually_by_date(day, month)?;
            let period_end = frequency.period_end(&period_start);

            Some(Self {
                frequency,
                period_start,
                period_end,
                grace_period_duration,
            })
        } else {
            None
        }
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
            grace_period_duration: self.grace_period_duration.clone(),
        }
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

    pub const fn period_end(&self) -> NaiveDate {
        self.period_end
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Year {
    Calendar,
    Fiscal { day: u32, month: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Month {
    Calendar,
    OnDay(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Frequency {
    Year(Year),
    Month(Month),
}

impl Frequency {
    pub const fn monthly_by_calendar() -> Self {
        Self::Month(Month::Calendar)
    }

    pub const fn monthly_by_day_in_month(day_in_month: u8) -> Option<Self> {
        if day_in_month > 1 && day_in_month < 28 {
            Some(Self::Month(Month::OnDay(day_in_month)))
        } else {
            None
        }
    }

    pub const fn annually_by_calendar() -> Self {
        Self::Year(Year::Calendar)
    }

    pub fn annually_by_date(day: u8, month: u8) -> Option<Self> {
        if !(day == 1 && month == 1) && day > 0 && day < 28 && month > 0 && month < 13 {
            Some(Self::Year(Year::Fiscal {
                day: u32::from(day),
                month: u32::from(month),
            }))
        } else {
            None
        }
    }

    pub const fn is_monthly(&self) -> bool {
        matches!(self, Frequency::Month(..))
    }

    pub const fn is_annual(&self) -> bool {
        matches!(self, Frequency::Year(..))
    }

    /// Returns end date of a period with this frequency and for given
    /// `period_start`
    pub fn period_end(&self, period_start: &NaiveDate) -> NaiveDate {
        match self {
            Frequency::Year(Year::Calendar) => period_start
                .with_year(period_start.year() + 1)
                .expect("January 1st is valid every year")
                .checked_sub_days(Days::new(1))
                .expect("always in valid date range"),
            Frequency::Year(Year::Fiscal { day, month }) => {
                NaiveDate::from_ymd_opt(period_start.year() + 1, *month, *day)
                    .expect("valid date")
                    .checked_sub_days(Days::new(1))
                    .expect("always in valid date range")
            }
            Frequency::Month(Month::Calendar) => period_start
                .with_day(period_start.num_days_in_month().into())
                .expect("always in valid date range"),
            Frequency::Month(Month::OnDay(d)) => period_start
                .with_day((*d).into())
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

    use super::{Frequency, Month, Year};

    fn dt(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn test(freq: &Frequency, start: &str, expected: &str) {
        assert_eq!(freq.period_end(&dt(start)), dt(expected));
    }

    #[test]
    fn frequency_calendar_month() {
        let freq = Frequency::monthly_by_calendar();

        test(&freq, "2025-05-01", "2025-05-31");
        test(&freq, "2025-04-01", "2025-04-30");
        test(&freq, "2025-03-01", "2025-03-31");
        test(&freq, "2025-12-01", "2025-12-31");
        test(&freq, "2025-01-01", "2025-01-31");
    }

    #[test]
    fn frequency_month_onday() {
        let freq = Frequency::monthly_by_day_in_month(12).unwrap();

        test(&freq, "2025-05-12", "2025-06-11");
        test(&freq, "2025-04-12", "2025-05-11");
        test(&freq, "2025-03-12", "2025-04-11");
        test(&freq, "2025-12-12", "2026-01-11");
        test(&freq, "2025-01-12", "2025-02-11");

        fn freq2(day: u8) -> Option<Frequency> {
            Frequency::monthly_by_day_in_month(day)
        }

        assert!(freq2(0).is_none());
        assert!(freq2(28).is_none());
    }

    #[test]
    fn frequency_calendar_year() {
        let freq = Frequency::annually_by_calendar();

        test(&freq, "2025-01-01", "2025-12-31");
        test(&freq, "2023-01-01", "2023-12-31");
    }

    #[test]
    fn frequency_fiscal_calendar() {
        fn freq(day: u8, month: u8) -> Frequency {
            Frequency::annually_by_date(day, month).unwrap()
        }

        test(&freq(1, 5), "2025-05-01", "2026-04-30");
        test(&freq(1, 4), "2025-04-01", "2026-03-31");
        test(&freq(1, 3), "2025-03-01", "2026-02-28");
        test(&freq(1, 12), "2025-12-01", "2026-11-30");

        fn freq2(day: u8, month: u8) -> Option<Frequency> {
            Frequency::annually_by_date(day, month)
        }

        assert!(freq2(1, 1).is_none());
        assert!(freq2(0, 4).is_none());
        assert!(freq2(4, 0).is_none());
        assert!(freq2(28, 10).is_none());
        assert!(freq2(10, 13).is_none());
    }
}
