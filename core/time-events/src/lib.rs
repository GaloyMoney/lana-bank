use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
mod error;

pub struct TimeEvents {
    config: TimeEventsConfig,
}

impl TimeEvents {
    pub async fn init() -> Self {
        let raw_config = RawTimeEventsConfig {
            timezone: "UTC".to_string(),
            closing_time: "00:00".to_string(),
        };

        let config = TimeEventsConfig::try_from(raw_config).expect("correct default config");

        Self { config }
    }

    fn next_closing_in_utc(self) -> DateTime<Utc> {
        let tz = self.config.timezone;
        let now_in_tz = Utc::now().with_timezone(&tz);
        let today = now_in_tz.date_naive();
        let naive_dt = today.and_time(self.config.closing_time);

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
        let timezone = Tz::from_str(value.timezone.as_str())?;
        let closing_time = NaiveTime::parse_from_str(&value.closing_time, "%H:%M:%S")?;

        Ok(TimeEventsConfig {
            timezone,
            closing_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
