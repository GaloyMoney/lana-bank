use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use es_entity::clock::ClockHandle;
#[derive(Clone, Debug)]
pub(crate) struct ClosingSchedule {
    timezone: Tz,
    next_closing: DateTime<Utc>,
}

impl ClosingSchedule {
    pub fn new(timezone: Tz, closing_time_of_day: NaiveTime, clock: &ClockHandle) -> Self {
        let current_time = clock.now();
        let next_closing =
            Self::calculate_next_closing(timezone, closing_time_of_day, current_time);
        Self {
            timezone,
            next_closing,
        }
    }

    pub fn next_closing(&self) -> DateTime<Utc> {
        self.next_closing
    }

    pub fn next_closing_day(&self) -> NaiveDate {
        self.next_closing.with_timezone(&self.timezone).date_naive()
    }

    pub fn closing_for_day(
        timezone: Tz,
        closing_time_of_day: NaiveTime,
        day: NaiveDate,
    ) -> DateTime<Utc> {
        let closing_naive_dt = day.and_time(closing_time_of_day);
        let time = match timezone.from_local_datetime(&closing_naive_dt) {
            chrono::LocalResult::Single(dt) => dt,
            chrono::LocalResult::Ambiguous(dt1, _) => dt1,
            chrono::LocalResult::None => timezone
                .from_local_datetime(&(closing_naive_dt + chrono::Duration::hours(1)))
                .earliest()
                .expect("time should always exist"),
        };
        time.with_timezone(&Utc)
    }

    fn calculate_next_closing(
        timezone: Tz,
        closing_time_of_day: NaiveTime,
        after_utc: DateTime<Utc>,
    ) -> DateTime<Utc> {
        let now_in_tz = after_utc.with_timezone(&timezone);
        let today = now_in_tz.date_naive();
        let mut closing_naive_dt = today.and_time(closing_time_of_day);

        // since we are returning "next_closing", <= will move to next day even if from_utc and closing_time are same
        if closing_naive_dt.time() <= now_in_tz.time() {
            closing_naive_dt = closing_naive_dt + chrono::Days::new(1)
        }

        let time = match timezone.from_local_datetime(&closing_naive_dt) {
            chrono::LocalResult::Single(dt) => dt,
            chrono::LocalResult::Ambiguous(dt1, dt2) => {
                // Pick whichever occurrence is in the future
                if dt1.with_timezone(&Utc) > after_utc {
                    dt1
                } else {
                    dt2
                }
            }
            // if the calculated "next closing time" does not exist, we currently add 1 hour
            chrono::LocalResult::None => timezone
                .from_local_datetime(&(closing_naive_dt + chrono::Duration::hours(1)))
                .earliest()
                .expect("time should always exist"),
        };

        time.with_timezone(&Utc)
    }
}

#[cfg(test)]
mod tests {
    use chrono::SecondsFormat;
    use es_entity::clock::ArtificialClockConfig;

    use super::*;

    fn clock_at(time: &str) -> ClockHandle {
        let (clock, _) =
            ClockHandle::artificial(ArtificialClockConfig::manual_at(time.parse().unwrap()));
        clock
    }

    #[test]
    fn next_closing_moves_to_next_day() {
        let clock = clock_at("2021-01-15T12:00:00Z");
        let schedule = ClosingSchedule::new(
            "UTC".parse().unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-01-16T00:00:00Z");
    }

    #[test]
    fn next_closing_same_day() {
        let clock = clock_at("2021-01-15T12:00:00Z");
        let schedule = ClosingSchedule::new(
            "UTC".parse().unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-01-15T18:00:00Z");
    }

    #[test]
    fn next_closing_with_est_offset() {
        // UTC - 5 in winters
        let clock = clock_at("2021-01-15T12:00:00Z");
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(13, 15, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-01-15T18:15:00Z");
    }

    #[test]
    fn next_closing_with_edt_offset() {
        // UTC - 4 in summer
        let clock = clock_at("2021-07-15T12:00:00Z");
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(13, 15, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-07-15T17:15:00Z");
    }

    #[test]
    fn next_closing_edt_next_day() {
        let clock = clock_at("2021-07-15T22:00:00Z");
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(13, 15, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        assert_eq!(next, "2021-07-16T17:15:00Z");
    }

    #[test]
    fn fallback_before_ambiguous_hour() {
        // Nov 7, 2021: Fall back happens at 2:00 AM → 1:00 AM in local time
        // Closing time is in the ambiguous hour (1:00-2:00 AM occurs twice)

        // from_utc: Nov 7, 2021 at 12:30 AM local time (before fall-back)
        // In UTC: Nov 7, 2021 at 4:30 AM
        let clock = clock_at("2021-11-07T04:30:00Z");
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // Should return 1:30 AM local time (first occurrence)
        assert_eq!(next, "2021-11-07T05:30:00Z");
    }

    #[test]
    fn fallback_during_second_occurrence() {
        // Nov 7, 2021: Fall back happens at 2:00 AM → 1:00 AM
        // Closing time is in the ambiguous hour (1:00-2:00 AM occurs twice)

        // Currently IN the "second" occurrence of the ambiguous hour
        // from_utc: Nov 7, 2021 at 1:15 AM EST not EDT (second occurrence, after fall-back)
        // In UTC: Nov 7, 2021 at 6:15 AM
        let clock = clock_at("2021-11-07T06:15:00Z");
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // Should return 6:30 AM UTC (second occurrence, 15 mins in future)
        assert_eq!(next, "2021-11-07T06:30:00Z");
    }

    #[test]
    fn fallback_past_first_occurrence() {
        // Nov 7, 2021: Fall back happens at 2:00 AM → 1:00 AM
        // Closing time is in the ambiguous hour (1:00-2:00 AM occurs twice)

        // Currently IN the first occurrence of the ambiguous hour, past the closing time
        // from_utc: Nov 7, 2021 at 1:45 AM EDT (first occurrence, before fall-back)
        // In UTC: Nov 7, 2021 at 5:45 AM
        let clock = clock_at("2021-11-07T05:45:00Z");
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // Current code skips the second occurrence (1:30 AM EST at 6:30 UTC)
        // and returns next days's closing at 6:30 AM UTC (1:30 AM EST next day)
        assert_eq!(next, "2021-11-08T06:30:00Z");
    }

    #[test]
    fn fallback_closing_after_ambiguous_hour() {
        // Nov 7, 2021: Fall back at 2:00 AM → 1:00 AM
        // Closing time is AFTER the ambiguous hour (1:00-2:00 AM)

        // from_utc: Nov 7, 2021 at 1:30 AM EDT (first occurrence in ambiguous hour)
        // In UTC: Nov 7, 2021 at 5:30 AM
        let clock = clock_at("2021-11-07T05:30:00Z");
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(5, 0, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // Closing at 5:00 AM local time/EST (after ambiguous period ends) = 10:00 AM UTC
        assert_eq!(next, "2021-11-07T10:00:00Z");
    }

    #[test]
    fn spring_forward_closing_in_gap() {
        // Mar 14, 2021: Spring forward at 2:00 AM → 3:00 AM
        // Closing time is in the gap (2:00-3:00 AM doesn't exist)

        // from_utc: Mar 14, 2021 at 1:00 AM EST (before spring forward)
        // In UTC: Mar 14, 2021 at 6:00 AM
        let clock = clock_at("2021-03-14T06:00:00Z");
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(2, 30, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // 2:30 AM doesn't exist, returns 3:30 AM EDT (after gap)
        // Current code adds 1 hour → 3:30 AM EDT = 7:30 AM UTC
        assert_eq!(next, "2021-03-14T07:30:00Z");
    }

    #[test]
    fn closing_for_day_normal() {
        let tz: Tz = "America/New_York".parse().unwrap();
        let closing_time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        let day = NaiveDate::from_ymd_opt(2021, 7, 15).unwrap();
        let result = ClosingSchedule::closing_for_day(tz, closing_time, day);
        // 5 PM EDT = 9 PM UTC
        assert_eq!(
            result.to_rfc3339_opts(SecondsFormat::Secs, true),
            "2021-07-15T21:00:00Z"
        );
    }

    #[test]
    fn closing_for_day_dst_gap() {
        // Mar 14, 2021: Spring forward at 2:00 AM → 3:00 AM in America/New_York
        let tz: Tz = "America/New_York".parse().unwrap();
        let closing_time = NaiveTime::from_hms_opt(2, 30, 0).unwrap();
        let day = NaiveDate::from_ymd_opt(2021, 3, 14).unwrap();
        let result = ClosingSchedule::closing_for_day(tz, closing_time, day);
        // 2:30 AM doesn't exist, shifted to 3:30 AM EDT = 7:30 AM UTC
        assert_eq!(
            result.to_rfc3339_opts(SecondsFormat::Secs, true),
            "2021-03-14T07:30:00Z"
        );
    }

    #[test]
    fn closing_for_day_ambiguous_hour() {
        // Nov 7, 2021: Fall back at 2:00 AM → 1:00 AM in America/New_York
        let tz: Tz = "America/New_York".parse().unwrap();
        let closing_time = NaiveTime::from_hms_opt(1, 30, 0).unwrap();
        let day = NaiveDate::from_ymd_opt(2021, 11, 7).unwrap();
        let result = ClosingSchedule::closing_for_day(tz, closing_time, day);
        // Ambiguous hour: picks first occurrence (EDT), 1:30 AM EDT = 5:30 AM UTC
        assert_eq!(
            result.to_rfc3339_opts(SecondsFormat::Secs, true),
            "2021-11-07T05:30:00Z"
        );
    }

    #[test]
    fn spring_forward_closing_after_gap() {
        // Mar 14, 2021: Spring forward at 2:00 AM → 3:00 AM
        // Closing time is after the gap

        // from_utc: Mar 14, 2021 at 1:00 AM EST (before spring forward)
        // In UTC: Mar 14, 2021 at 6:00 AM
        let clock = clock_at("2021-03-14T06:00:00Z");
        let schedule = ClosingSchedule::new(
            "America/New_York".parse().unwrap(),
            NaiveTime::from_hms_opt(3, 30, 0).unwrap(),
            &clock,
        );
        let next = schedule
            .next_closing()
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        // 3:30 AM EDT = 7:30 AM UTC
        assert_eq!(next, "2021-03-14T07:30:00Z");
    }
}
