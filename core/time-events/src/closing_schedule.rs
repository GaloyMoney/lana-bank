use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;

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
            // even if called for time in second occurrence, which will probably change, also shown in test
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
