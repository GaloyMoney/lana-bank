use std::str::FromStr;

use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use domain_config::{DomainConfigError, DomainConfigKey, DomainConfigValue, DomainConfigs};
use serde::{Deserialize, Serialize};

use crate::error::TimeEventsError;
mod error;
mod time;

pub trait Now {
    fn now(&self) -> DateTime<Utc>;
}

pub struct RealNow;

impl Now for RealNow {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub struct TimeEvents<T: Now> {
    domain_configs: DomainConfigs,
    // config: TimeEventsConfig,
    now_fn: T,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TimezoneConfig {
    // pub timezone_db: String
    // we need a way to convert (serde) the value in the db into another type 
    // after it being loaded.
    // ie: here for timezone we want a Tz type, for closing_time we want a NaiveTime
    // I think this is what the domainConfig API should expose automatically after loading the value

    pub timezone: Tz,
}

impl DomainConfigValue for TimezoneConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("timezone");

    fn validate(&self) -> Result<(), DomainConfigError> {
        get_timezone(self.timezone.clone())
            .map_err(|e| DomainConfigError::InvalidState("invalid timezone".to_string()))?;

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
        get_closing_time(self.closing_time.clone())
            .map_err(|e| DomainConfigError::InvalidState("invalid timezone".to_string()))?;

        Ok(())
    }
}

impl<T: Now> TimeEvents<T> {
    pub fn init(domain_configs: DomainConfigs, now_fn: T) -> Result<Self, TimeEventsError> {
        let _test = now_fn.now();

        // let config = TimeEventsConfig::try_from(raw_config)?;

        Ok(Self {
            domain_configs,
            now_fn,
        })
    }

    async fn next_closing_in_utc(self) -> DateTime<Utc> {
        let tz = self
            .domain_configs
            .get_or_default::<TimezoneConfig>()
            .await?;

        let closing_time = self
            .domain_configs
            .get_or_default::<ClosingTimeConfig>()
            .await?;

        let now_in_tz = self.now_fn.now().with_timezone(&tz);
        let today = now_in_tz.date_naive();
        let mut naive_dt = today.and_time(closing_time);

        if naive_dt.time() < now_in_tz.time() {
            naive_dt = naive_dt + chrono::Days::new(1)
        }

        let time = match tz.from_local_datetime(&naive_dt) {
            chrono::LocalResult::Single(dt) => dt,
            chrono::LocalResult::Ambiguous(dt1, _) => dt1, // pick earliest
            chrono::LocalResult::None => tz
                .from_local_datetime(&(naive_dt + chrono::Duration::hours(1)))
                .earliest()
                .expect("time should always exist"),
        };

        let time_utc = Utc.timestamp_opt(time.timestamp(), time.timestamp_subsec_nanos());

        match time_utc {
            chrono::offset::LocalResult::Single(time) => time,
            _ => panic!("there should always be a single time"),
        }

        // let maybe_next_closing = Utc.from
    }

    // async load_config(rawTime: RawTimeEventsConfig): TimeEventsConfig {
    //     TimeEventsConfig {
    //         closing_time: DateTime
    //     }
    // }
}

pub struct TimeEventsConfig {
    timezone: Tz,
    closing_time: NaiveTime,
}

struct RawTimeEventsConfig {
    timezone: String,
    closing_time: String,
}

impl TryFrom<RawTimeEventsConfig> for TimeEventsConfig {
    type Error = error::TimeEventsError;

    fn try_from(value: RawTimeEventsConfig) -> Result<Self, Self::Error> {
        let timezone = get_timezone(value.timezone)?;
        let closing_time = get_closing_time(value.closing_time)?;

        Ok(TimeEventsConfig {
            timezone,
            closing_time,
        })
    }
}

fn get_timezone(timezone: String) -> Result<chrono_tz::Tz, TimeEventsError> {
    Ok(Tz::from_str(timezone.as_str())?)
}

fn get_closing_time(time: String) -> Result<NaiveTime, TimeEventsError> {
    Ok(NaiveTime::parse_from_str(&time, "%H:%M:%S")?)
}

#[cfg(test)]
mod tests {
    struct MockNow {
        date_raw: String,
    }

    impl Now for MockNow {
        fn now(&self) -> DateTime<Utc> {
            self.date_raw.parse::<DateTime<Utc>>().unwrap()
        }
    }

    use chrono::SecondsFormat;

    use super::*;

    #[test]
    fn error_with_wrong_config() {
        // here I want to do something like:
        let preCachedDomainConfig = DomainConfigPreCached::init(Cache {
            timezone: "UTC".to_string(),
            closing_time: "00:11:22:33".to_string(),
        });

        // so that I don't have to set up an integration test for this module
        // rather I just want to do unit test
        //
        // currently this is not possible because domain config needs a database connection
        // but this make every test that relies on domain an integration currently
        //
        // this is a poor DX and will make test a lot longer to run


        // let raw_config = RawTimeEventsConfig {
        //     timezone: "UTC".to_string(),
        //     closing_time: "00:11:22:33".to_string(),
        // };

        let time_events = TimeEvents::init(
            preCachedDomainConfig,
            MockNow {
                date_raw: "2021-01-15T12:00:00Z".to_string(),
            },
        );

        assert!(time_events.is_err());
    }

    #[test]
    fn test_with_simple_config() {
        let time_events = TimeEvents::init(
            RawTimeEventsConfig {
                timezone: "UTC".to_string(),
                closing_time: "00:00:00".to_string(),
            },
            MockNow {
                date_raw: "2021-01-15T12:00:00Z".to_string(),
                // date_raw: "2021-01-15 12:00:00".to_string(),
            },
        );

        assert!(time_events.is_ok());
    }

    #[test]
    fn calculate_next_closing_after_hours() {
        let time_events = TimeEvents::init(
            RawTimeEventsConfig {
                timezone: "UTC".to_string(),
                closing_time: "00:00:00".to_string(),
            },
            MockNow {
                date_raw: "2021-01-15T12:00:00Z".to_string(),
            },
        );

        let next_event = time_events
            .unwrap()
            .next_closing_in_utc()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next_event, "2021-01-16T00:00:00Z");
    }

    #[test]
    fn calculate_next_closing_before_hours() {
        let time_events = TimeEvents::init(
            RawTimeEventsConfig {
                timezone: "UTC".to_string(),
                closing_time: "18:00:00".to_string(),
            },
            MockNow {
                date_raw: "2021-01-15T12:00:00Z".to_string(),
            },
        );

        let next_event = time_events
            .unwrap()
            .next_closing_in_utc()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next_event, "2021-01-15T18:00:00Z");
    }

    #[test]
    fn calculate_next_closing_timezone_nyc_winter() {
        let time_events = TimeEvents::init(
            RawTimeEventsConfig {
                timezone: "America/New_York".to_string(),
                closing_time: "13:15:00".to_string(),
            },
            MockNow {
                date_raw: "2021-01-15T12:00:00Z".to_string(),
            },
        );

        let next_event = time_events
            .unwrap()
            .next_closing_in_utc()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // 12h + 1h15 + 5h winter
        assert_eq!(next_event, "2021-01-15T18:15:00Z");
    }
    #[test]
    fn calculate_next_closing_timezone_nyc_summer() {
        let time_events = TimeEvents::init(
            RawTimeEventsConfig {
                timezone: "America/New_York".to_string(),
                closing_time: "13:15:00".to_string(),
            },
            MockNow {
                date_raw: "2021-07-15T12:00:00Z".to_string(),
            },
        );

        let next_event = time_events
            .unwrap()
            .next_closing_in_utc()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // 12h + 1h15 + 4h summer
        assert_eq!(next_event, "2021-07-15T17:15:00Z");
    }

    #[test]
    fn calculate_next_closing_timezone_nyc_summer_past_time() {
        let time_events = TimeEvents::init(
            RawTimeEventsConfig {
                timezone: "America/New_York".to_string(),
                closing_time: "13:15:00".to_string(),
            },
            MockNow {
                date_raw: "2021-07-15T22:00:00Z".to_string(),
            },
        );

        let next_event = time_events
            .unwrap()
            .next_closing_in_utc()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next_event, "2021-07-16T17:15:00Z");
    }
}
