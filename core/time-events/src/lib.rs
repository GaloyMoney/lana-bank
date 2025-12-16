use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use domain_config::{DomainConfigError, DomainConfigKey, DomainConfigValue, DomainConfigs};
use serde::{Deserialize, Serialize};

use crate::error::TimeEventsError;
mod error;

pub trait Now {
    fn now(&self) -> DateTime<Utc>;
}

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
        let mut naive_dt = today.and_time(self.closing_time);

        if naive_dt.time() < now_in_tz.time() {
            naive_dt = naive_dt + chrono::Days::new(1)
        }

        let time = match self.timezone.from_local_datetime(&naive_dt) {
            chrono::LocalResult::Single(dt) => dt,
            chrono::LocalResult::Ambiguous(dt1, _) => dt1, // pick earliest
            chrono::LocalResult::None => self
                .timezone
                .from_local_datetime(&(naive_dt + chrono::Duration::hours(1)))
                .earliest()
                .expect("time should always exist"),
        };

        let time_utc = Utc.timestamp_opt(time.timestamp(), time.timestamp_subsec_nanos());

        match time_utc {
            chrono::offset::LocalResult::Single(time) => time,
            _ => panic!("there should always be a single time"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimezoneConfig {
    pub timezone: Tz,
}

impl Default for TimezoneConfig {
    fn default() -> Self {
        Self { timezone: chrono_tz::UTC }
    }
}

impl DomainConfigValue for TimezoneConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("timezone");

    fn validate(&self) -> Result<(), DomainConfigError> {
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClosingTimeConfig {
    pub closing_time: NaiveTime,
}

impl Default for ClosingTimeConfig {
    fn default() -> Self {
        Self {
            closing_time: NaiveTime::from_hms_opt(0, 0, 0).expect("valid time"),
        }
    }
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
    fn calculate_next_closing_timezone_nyc_winter() {
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
    fn calculate_next_closing_timezone_nyc_summer() {
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
    fn calculate_next_closing_timezone_nyc_summer_past_time() {
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(13, 15, 0).unwrap(),
        );
        let next = schedule
            .next_closing_from("2021-07-15T22:00:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-07-16T17:15:00Z");
    }
}
