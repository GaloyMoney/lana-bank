mod error;
mod event;
mod jobs;

use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use domain_config::{DomainConfigError, DomainConfigKey, DomainConfigValue, DomainConfigs};

use crate::error::TimeEventsError;

use event::*;
// structure of imports/declarations

pub trait Now {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Clone)]
pub struct RealNow;

impl Now for RealNow {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosingSchedule {
    pub timezone: Tz,
    pub closing_time: NaiveTime,
}

impl ClosingSchedule {
    pub fn new(timezone: Tz, closing_time: NaiveTime) -> Self {
        Self {
            timezone,
            closing_time,
        }
    }

    /// Returns the next closing time after `from_utc` expressed in UTC.
    pub fn next_closing_from(&self, from_utc: DateTime<Utc>) -> DateTime<Utc> {
        let now_in_tz = from_utc.with_timezone(&self.timezone);
        let today = now_in_tz.date_naive();
        let mut closing_naive_dt = today.and_time(self.closing_time);
        // If the from_utc and closing time are same, do we not want next day instead of same time?
        // since we are returning "next_closing", which answers do we want <= instead?
        if closing_naive_dt.time() < now_in_tz.time() {
            closing_naive_dt = closing_naive_dt + chrono::Days::new(1)
        }

        let time = match self.timezone.from_local_datetime(&closing_naive_dt) {
            chrono::LocalResult::Single(dt) => dt,
            // if from_utc < closing_time and both lie in ambiguous window, we get the past/earliest/dt1 closing time
            // even if called for time in second occurrence, shown in test
            // do we not want "next_closing" instead past time
            chrono::LocalResult::Ambiguous(dt1, _) => dt1,
            // pick earliest
            chrono::LocalResult::None => self
                .timezone
                .from_local_datetime(&(closing_naive_dt + chrono::Duration::hours(1)))
                .earliest()
                .expect("time should always exist"),
        };

        time.with_timezone(&Utc)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TimezoneConfig {
    pub timezone: Tz,
}

impl DomainConfigValue for TimezoneConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("timezone");

    fn validate(&self) -> Result<(), DomainConfigError> {
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ClosingTimeConfig {
    pub closing_time: NaiveTime,
}

impl DomainConfigValue for ClosingTimeConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("closing-time");

    fn validate(&self) -> Result<(), DomainConfigError> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct TimeEvents<T: Now> {
    domain_configs: DomainConfigs,
    now_fn: T,
}

impl<T: Now> TimeEvents<T> {
    pub fn init(domain_configs: DomainConfigs, now_fn: T) -> Self {
        Self {
            domain_configs,
            now_fn,
        }
    }

    pub async fn next_closing_in_utc(&self) -> Result<DateTime<Utc>, TimeEventsError> {
        let tz_config = self
            .domain_configs
            .get_or_default::<TimezoneConfig>()
            .await?;

        let closing_time_config = self
            .domain_configs
            .get_or_default::<ClosingTimeConfig>()
            .await?;

        let schedule = ClosingSchedule::new(tz_config.timezone, closing_time_config.closing_time);

        Ok(schedule.next_closing_from(self.now_fn.now()))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveTime, SecondsFormat};

    use super::*;

    #[test]
    fn calculate_next_closing_after_hours() {
        let schedule = ClosingSchedule::new(
            "UTC".parse().unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        );
        let next = schedule
            .next_closing_from("2021-01-15T12:00:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-01-16T00:00:00Z");
    }

    #[test]
    fn calculate_next_closing_before_hours() {
        let schedule = ClosingSchedule::new(
            "UTC".parse().unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        );
        let next = schedule
            .next_closing_from("2021-01-15T12:00:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-01-15T18:00:00Z");
    }

    #[test]
    fn calculate_next_closing_nyc_winter() {
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(13, 15, 0).unwrap(),
        );
        let next = schedule
            .next_closing_from("2021-01-15T12:00:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-01-15T18:15:00Z");
    }

    #[test]
    fn calculate_next_closing_nyc_summer() {
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(13, 15, 0).unwrap(),
        );
        let next = schedule
            .next_closing_from("2021-07-15T12:00:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-07-15T17:15:00Z");
    }

    #[test]
    fn calculate_next_closing_nyc_summer_after_hours() {
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(13, 15, 0).unwrap(),
        );
        let next = schedule
            .next_closing_from("2021-07-15T22:00:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-07-16T17:15:00Z");
    }

    #[test]
    fn calculate_next_closing_fall_back_before_ambiguous() {
        // Nov 7, 2021: Fall back happens at 2:00 AM → 1:00 AM
        // Closing time is in the ambiguous hour (1:00-2:00 AM occurs twice)
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
        );

        // from_utc: Nov 7, 2021 at 12:30 AM EDT (before fall-back)
        // In UTC: Nov 7, 2021 at 4:30 AM
        let next = schedule
            .next_closing_from("2021-11-07T04:30:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // Should return 1:30 AM EDT (first occurrence)
        assert_eq!(next, "2021-11-07T05:30:00Z");
    }

    #[test]
    fn calculate_next_closing_fall_back_during_ambiguous() {
        // Nov 7, 2021: Fall back happens at 2:00 AM → 1:00 AM
        // Closing time is in the ambiguous hour (1:00-2:00 AM occurs twice)
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
        );

        // Currently IN the second occurrence of the ambiguous hour
        // from_utc: Nov 7, 2021 at 1:15 AM EST (second occurrence, after fall-back)
        // In UTC: Nov 7, 2021 at 6:15 AM
        let next = schedule
            .next_closing_from("2021-11-07T06:15:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // Current code returns 5:30 AM UTC (first occurrence, which is 45mins in the past)
        assert_eq!(next, "2021-11-07T05:30:00Z");
    }

    #[test]
    fn calculate_next_closing_fall_back_skips_second_occurrence() {
        // Nov 7, 2021: Fall back happens at 2:00 AM → 1:00 AM
        // Closing time is in the ambiguous hour (1:00-2:00 AM occurs twice)
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
        );

        // Currently IN the first occurrence of the ambiguous hour, past the closing time
        // from_utc: Nov 7, 2021 at 1:45 AM EDT (first occurrence, before fall-back)
        // In UTC: Nov 7, 2021 at 5:45 AM
        let next = schedule
            .next_closing_from("2021-11-07T05:45:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // Current code skips the second occurrence (1:30 AM EST at 6:30 UTC)
        // and returns next days's closing at 6:30 AM UTC (1:30 AM EST next day)
        assert_eq!(next, "2021-11-08T06:30:00Z");
    }

    #[test]
    fn calculate_next_closing_fall_back_closing_after_ambiguous() {
        // Nov 7, 2021: Fall back at 2:00 AM → 1:00 AM
        // Closing time is AFTER the ambiguous hour (1:00-2:00 AM)
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(5, 0, 0).unwrap(),
        );

        // from_utc: Nov 7, 2021 at 1:30 AM EDT (first occurrence in ambiguous hour)
        // In UTC: Nov 7, 2021 at 5:30 AM
        let next = schedule
            .next_closing_from("2021-11-07T05:30:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // Closing at 5:00 AM EST (after ambiguous period ends) = 10:00 AM UTC
        assert_eq!(next, "2021-11-07T10:00:00Z");
    }

    #[test]
    fn calculate_next_closing_spring_forward_closing_in_gap() {
        // Mar 14, 2021: Spring forward at 2:00 AM → 3:00 AM
        // Closing time is in the gap (2:00-3:00 AM doesn't exist)
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(2, 30, 0).unwrap(),
        );

        // from_utc: Mar 14, 2021 at 1:00 AM EST (before spring forward)
        // In UTC: Mar 14, 2021 at 6:00 AM
        let next = schedule
            .next_closing_from("2021-03-14T06:00:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // 2:30 AM doesn't exist, returns 3:30 AM EDT (after gap)
        // Current code adds 1 hour → 3:30 AM EDT = 7:30 AM UTC
        assert_eq!(next, "2021-03-14T07:30:00Z");
    }

    #[test]
    fn calculate_next_closing_spring_forward_closing_after_gap() {
        // Mar 14, 2021: Spring forward at 2:00 AM → 3:00 AM
        // Closing time is after the gap
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(3, 30, 0).unwrap(),
        );

        // from_utc: Mar 14, 2021 at 1:00 AM EST (before spring forward)
        // In UTC: Mar 14, 2021 at 6:00 AM
        let next = schedule
            .next_closing_from("2021-03-14T06:00:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // 3:30 AM EDT = 7:30 AM UTC
        assert_eq!(next, "2021-03-14T07:30:00Z");
    }
}
