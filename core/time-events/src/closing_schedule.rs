use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ClosingSchedule {
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

        // since we are returning "next_closing", <= will move to next day even if from_utc and closing_time are same
        if closing_naive_dt.time() <= now_in_tz.time() {
            closing_naive_dt = closing_naive_dt + chrono::Days::new(1)
        }

        let time = match self.timezone.from_local_datetime(&closing_naive_dt) {
            chrono::LocalResult::Single(dt) => dt,
            chrono::LocalResult::Ambiguous(dt1, dt2) => {
                // Pick whichever occurrence is in the future
                if dt1.with_timezone(&Utc) > from_utc {
                    dt1
                } else {
                    dt2
                }
            }
            // if the calculated "next closing time" does not exist, we currently add 1 hour
            chrono::LocalResult::None => self
                .timezone
                .from_local_datetime(&(closing_naive_dt + chrono::Duration::hours(1)))
                .earliest()
                .expect("time should always exist"),
        };

        time.with_timezone(&Utc)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveTime, SecondsFormat};

    use super::*;

    #[test]
    fn next_closing_moves_to_next_day() {
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
    fn next_closing_same_day() {
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
    fn next_closing_with_est_offset() {
        // UTC - 5 in winters
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
    fn next_closing_with_edt_offset() {
        // UTC - 4 in summer
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
    fn next_closing_edt_next_day() {
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
    fn fallback_before_ambiguous_hour() {
        // Nov 7, 2021: Fall back happens at 2:00 AM → 1:00 AM in local time
        // Closing time is in the ambiguous hour (1:00-2:00 AM occurs twice)
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
        );

        // from_utc: Nov 7, 2021 at 12:30 AM local time (before fall-back)
        // In UTC: Nov 7, 2021 at 4:30 AM
        let next = schedule
            .next_closing_from("2021-11-07T04:30:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // Should return 1:30 AM local time (first occurrence)
        assert_eq!(next, "2021-11-07T05:30:00Z");
    }

    #[test]
    fn fallback_during_second_occurrence() {
        // Nov 7, 2021: Fall back happens at 2:00 AM → 1:00 AM
        // Closing time is in the ambiguous hour (1:00-2:00 AM occurs twice)
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
        );

        // Currently IN the "second" occurrence of the ambiguous hour
        // from_utc: Nov 7, 2021 at 1:15 AM EST not EDT (second occurrence, after fall-back)
        // In UTC: Nov 7, 2021 at 6:15 AM
        let next = schedule
            .next_closing_from("2021-11-07T06:15:00Z".parse().unwrap())
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // Should return 6:30 AM UTC (second occurrence, 15 mins in future)
        assert_eq!(next, "2021-11-07T06:30:00Z");
    }

    #[test]
    fn fallback_past_first_occurrence() {
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
    fn fallback_closing_after_ambiguous_hour() {
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

        // Closing at 5:00 AM local time/EST (after ambiguous period ends) = 10:00 AM UTC
        assert_eq!(next, "2021-11-07T10:00:00Z");
    }

    #[test]
    fn spring_forward_closing_in_gap() {
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
    fn spring_forward_closing_after_gap() {
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
