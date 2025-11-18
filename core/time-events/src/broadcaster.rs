use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use tracing::{error, info, instrument};

#[cfg(test)]
use chrono::{Datelike, Timelike};

use outbox::{EphemeralEventType, Outbox, OutboxEventMarker};

use crate::{config::TimeEventsConfig, error::TimeEventsError, event::TimeEvent};

pub struct DailyClosingBroadcaster<E>
where
    E: OutboxEventMarker<TimeEvent>,
{
    outbox: Outbox<E>,
    config: TimeEventsConfig,
}

impl<E> DailyClosingBroadcaster<E>
where
    E: OutboxEventMarker<TimeEvent>,
{
    pub fn new(outbox: &Outbox<E>, config: TimeEventsConfig) -> Self {
        Self {
            outbox: outbox.clone(),
            config,
        }
    }

    pub fn outbox(&self) -> &Outbox<E> {
        &self.outbox
    }

    pub fn config(&self) -> &TimeEventsConfig {
        &self.config
    }
}

impl<E> DailyClosingBroadcaster<E>
where
    E: OutboxEventMarker<TimeEvent> + Send + Sync + 'static,
{
    fn parse_timezone(&self) -> Result<Tz, TimeEventsError> {
        self.config
            .daily
            .timezone
            .parse()
            .map_err(|_| TimeEventsError::InvalidTimezone {
                timezone: self.config.daily.timezone.clone(),
            })
    }

    fn parse_closing_time(&self) -> Result<NaiveTime, TimeEventsError> {
        NaiveTime::parse_from_str(&self.config.daily.closing_time, "%H:%M:%S").map_err(|_| {
            TimeEventsError::InvalidTimeFormat {
                time_format: self.config.daily.closing_time.clone(),
            }
        })
    }

    fn calculate_next_closing(&self, now: DateTime<Utc>) -> Result<DateTime<Tz>, TimeEventsError> {
        let tz = self.parse_timezone()?;
        let closing_time = self.parse_closing_time()?;

        let now_in_tz = now.with_timezone(&tz);

        // Handle DST transitions when resolving local datetime
        let today_closing =
            match tz.from_local_datetime(&now_in_tz.date_naive().and_time(closing_time)) {
                chrono::LocalResult::Single(dt) => dt,
                // During "spring forward" gap, use the time after the gap
                chrono::LocalResult::None => {
                    // The time doesn't exist, so we add the duration to find the next valid time
                    let naive_dt = now_in_tz.date_naive().and_time(closing_time);
                    tz.from_local_datetime(&(naive_dt + chrono::Duration::hours(1)))
                        .earliest()
                        .ok_or_else(|| TimeEventsError::InvalidClosingDateTime {
                            closing_time: closing_time.to_string(),
                        })?
                }
                // During "fall back" overlap, use the earlier occurrence (first pass)
                chrono::LocalResult::Ambiguous(earlier, _later) => earlier,
            };

        let next_closing = if now_in_tz < today_closing {
            today_closing
        } else {
            let tomorrow = now_in_tz.date_naive() + chrono::Duration::days(1);
            match tz.from_local_datetime(&tomorrow.and_time(closing_time)) {
                chrono::LocalResult::Single(dt) => dt,
                // During "spring forward" gap, use the time after the gap
                chrono::LocalResult::None => {
                    let naive_dt = tomorrow.and_time(closing_time);
                    tz.from_local_datetime(&(naive_dt + chrono::Duration::hours(1)))
                        .earliest()
                        .ok_or_else(|| TimeEventsError::InvalidClosingDateTime {
                            closing_time: closing_time.to_string(),
                        })?
                }
                // During "fall back" overlap, use the earlier occurrence (first pass)
                chrono::LocalResult::Ambiguous(earlier, _later) => earlier,
            }
        };

        Ok(next_closing)
    }

    #[instrument(
        name = "time_events.broadcaster.publish_daily_closing",
        skip(self),
        fields(closing_time = %closing_time.to_rfc3339(), timezone = %closing_time.timezone()),
        err
    )]
    async fn publish_daily_closing(
        &self,
        closing_time: DateTime<Tz>,
    ) -> Result<(), TimeEventsError> {
        // Convert to UTC for the event - consumers only need the timestamp
        let closing_time_utc = closing_time.with_timezone(&Utc);

        self.outbox
            .publish_ephemeral(
                EphemeralEventType::new("time.daily-closing"),
                TimeEvent::DailyClosing {
                    closing_time: closing_time_utc,
                },
            )
            .await?;
        Ok(())
    }

    #[instrument(name = "time_events.broadcaster.run", skip(self))]
    pub async fn run(self) {
        info!(
            closing_time = %self.config.daily.closing_time,
            timezone = %self.config.daily.timezone,
            "Starting DailyClosing broadcaster"
        );

        // Validate timezone at startup - it won't change during runtime
        if let Err(e) = self.parse_timezone() {
            error!(error = %e, "Failed to parse timezone, broadcaster cannot start");
            return;
        }

        loop {
            let now = crate::time::now();

            match self.calculate_next_closing(now) {
                Ok(next_closing) => {
                    let duration = next_closing.signed_duration_since(now);

                    if let Ok(std_duration) = duration.to_std() {
                        info!(
                            next_closing = %next_closing,
                            sleep_duration_secs = %std_duration.as_secs(),
                            "Waiting until next closing time"
                        );

                        tokio::time::sleep(std_duration).await;

                        loop {
                            match self.publish_daily_closing(next_closing).await {
                                Ok(()) => {
                                    break;
                                }
                                Err(e) => {
                                    error!(error = %e, "Failed to publish DailyClosing event");
                                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                                }
                            }
                        }
                    } else {
                        error!("Duration calculation resulted in negative value, waiting 1 minute");
                        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to calculate next closing time, waiting 1 minute");
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DailyConfig;

    // Test struct that wraps only the config for testing calculation logic
    struct TestBroadcaster {
        config: TimeEventsConfig,
    }

    impl TestBroadcaster {
        fn new(closing_time: &str, timezone: &str) -> Self {
            Self {
                config: TimeEventsConfig {
                    daily: DailyConfig {
                        closing_time: closing_time.to_string(),
                        timezone: timezone.to_string(),
                    },
                },
            }
        }

        fn parse_timezone(&self) -> Result<Tz, TimeEventsError> {
            self.config
                .daily
                .timezone
                .parse()
                .map_err(|_| TimeEventsError::InvalidTimezone {
                    timezone: self.config.daily.timezone.clone(),
                })
        }

        fn parse_closing_time(&self) -> Result<NaiveTime, TimeEventsError> {
            NaiveTime::parse_from_str(&self.config.daily.closing_time, "%H:%M:%S").map_err(|_| {
                TimeEventsError::InvalidTimeFormat {
                    time_format: self.config.daily.closing_time.clone(),
                }
            })
        }

        fn calculate_next_closing(
            &self,
            now: DateTime<Utc>,
        ) -> Result<DateTime<Tz>, TimeEventsError> {
            let tz = self.parse_timezone()?;
            let closing_time = self.parse_closing_time()?;

            let now_in_tz = now.with_timezone(&tz);

            // Handle DST transitions when resolving local datetime
            let today_closing =
                match tz.from_local_datetime(&now_in_tz.date_naive().and_time(closing_time)) {
                    chrono::LocalResult::Single(dt) => dt,
                    // During "spring forward" gap, use the time after the gap
                    chrono::LocalResult::None => {
                        let naive_dt = now_in_tz.date_naive().and_time(closing_time);
                        tz.from_local_datetime(&(naive_dt + chrono::Duration::hours(1)))
                            .earliest()
                            .ok_or_else(|| TimeEventsError::InvalidClosingDateTime {
                                closing_time: closing_time.to_string(),
                            })?
                    }
                    // During "fall back" overlap, use the earlier occurrence (first pass)
                    chrono::LocalResult::Ambiguous(earlier, _later) => earlier,
                };

            let next_closing = if now_in_tz < today_closing {
                today_closing
            } else {
                let tomorrow = now_in_tz.date_naive() + chrono::Duration::days(1);
                match tz.from_local_datetime(&tomorrow.and_time(closing_time)) {
                    chrono::LocalResult::Single(dt) => dt,
                    // During "spring forward" gap, use the time after the gap
                    chrono::LocalResult::None => {
                        let naive_dt = tomorrow.and_time(closing_time);
                        tz.from_local_datetime(&(naive_dt + chrono::Duration::hours(1)))
                            .earliest()
                            .ok_or_else(|| TimeEventsError::InvalidClosingDateTime {
                                closing_time: closing_time.to_string(),
                            })?
                    }
                    // During "fall back" overlap, use the earlier occurrence (first pass)
                    chrono::LocalResult::Ambiguous(earlier, _later) => earlier,
                }
            };

            Ok(next_closing)
        }
    }

    #[test]
    fn test_before_closing_time() {
        let broadcaster = TestBroadcaster::new("23:59:00", "UTC");

        // Current time: 2024-01-15 10:00:00 UTC (before closing)
        let now = Utc
            .with_ymd_and_hms(2024, 1, 15, 10, 0, 0)
            .single()
            .unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2024-01-15 23:59:00 UTC (today's closing)
        let expected = Utc
            .with_ymd_and_hms(2024, 1, 15, 23, 59, 0)
            .single()
            .unwrap();

        assert_eq!(next_closing.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_after_closing_time() {
        let broadcaster = TestBroadcaster::new("23:59:00", "UTC");

        // Current time: 2024-01-15 23:59:01 UTC (after closing)
        let now = Utc
            .with_ymd_and_hms(2024, 1, 15, 23, 59, 1)
            .single()
            .unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2024-01-16 23:59:00 UTC (tomorrow's closing)
        let expected = Utc
            .with_ymd_and_hms(2024, 1, 16, 23, 59, 0)
            .single()
            .unwrap();

        assert_eq!(next_closing.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_exactly_at_closing_time() {
        let broadcaster = TestBroadcaster::new("23:59:00", "UTC");

        // Current time: 2024-01-15 23:59:00 UTC (exactly at closing)
        let now = Utc
            .with_ymd_and_hms(2024, 1, 15, 23, 59, 0)
            .single()
            .unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2024-01-16 23:59:00 UTC (tomorrow's closing, since we're not < closing time)
        let expected = Utc
            .with_ymd_and_hms(2024, 1, 16, 23, 59, 0)
            .single()
            .unwrap();

        assert_eq!(next_closing.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_timezone_america_new_york() {
        let broadcaster = TestBroadcaster::new("17:00:00", "America/New_York");

        // Current time: 2024-01-15 20:00:00 UTC (15:00 EST, before 17:00 EST closing)
        let now = Utc
            .with_ymd_and_hms(2024, 1, 15, 20, 0, 0)
            .single()
            .unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2024-01-15 22:00:00 UTC (17:00 EST = 22:00 UTC in winter)
        let expected = Utc
            .with_ymd_and_hms(2024, 1, 15, 22, 0, 0)
            .single()
            .unwrap();

        assert_eq!(next_closing.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_timezone_asia_tokyo() {
        let broadcaster = TestBroadcaster::new("18:00:00", "Asia/Tokyo");

        // Current time: 2024-01-15 08:00:00 UTC (17:00 JST, before 18:00 JST closing)
        let now = Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).single().unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2024-01-15 09:00:00 UTC (18:00 JST = 09:00 UTC)
        let expected = Utc.with_ymd_and_hms(2024, 1, 15, 9, 0, 0).single().unwrap();

        assert_eq!(next_closing.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_edge_case_closing_times() {
        // Test midnight closing (before)
        let now = Utc
            .with_ymd_and_hms(2024, 1, 15, 23, 0, 0)
            .single()
            .unwrap();
        let next = TestBroadcaster::new("00:00:00", "UTC")
            .calculate_next_closing(now)
            .unwrap();
        assert_eq!(
            next.with_timezone(&Utc),
            Utc.with_ymd_and_hms(2024, 1, 16, 0, 0, 0).single().unwrap()
        );

        // Test midnight closing (after)
        let now = Utc.with_ymd_and_hms(2024, 1, 16, 0, 0, 1).single().unwrap();
        let next = TestBroadcaster::new("00:00:00", "UTC")
            .calculate_next_closing(now)
            .unwrap();
        assert_eq!(
            next.with_timezone(&Utc),
            Utc.with_ymd_and_hms(2024, 1, 17, 0, 0, 0).single().unwrap()
        );

        // Test noon closing
        let now = Utc
            .with_ymd_and_hms(2024, 1, 15, 11, 30, 0)
            .single()
            .unwrap();
        let next = TestBroadcaster::new("12:00:00", "UTC")
            .calculate_next_closing(now)
            .unwrap();
        assert_eq!(
            next.with_timezone(&Utc),
            Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0)
                .single()
                .unwrap()
        );
    }

    #[test]
    fn test_invalid_timezone() {
        let broadcaster = TestBroadcaster::new("12:00:00", "Invalid/Timezone");

        let now = Utc::now();
        let result = broadcaster.calculate_next_closing(now);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TimeEventsError::InvalidTimezone { .. }
        ));
    }

    #[test]
    fn test_invalid_time_formats() {
        let now = Utc::now();

        // Test invalid hour
        let result = TestBroadcaster::new("25:00:00", "UTC").calculate_next_closing(now);
        assert!(matches!(
            result.unwrap_err(),
            TimeEventsError::InvalidTimeFormat { .. }
        ));

        // Test wrong pattern
        let result = TestBroadcaster::new("12:00", "UTC").calculate_next_closing(now);
        assert!(matches!(
            result.unwrap_err(),
            TimeEventsError::InvalidTimeFormat { .. }
        ));
    }

    #[test]
    fn test_month_boundary() {
        let broadcaster = TestBroadcaster::new("23:59:00", "UTC");

        // Current time: 2024-01-31 23:59:01 UTC (end of month, after closing)
        let now = Utc
            .with_ymd_and_hms(2024, 1, 31, 23, 59, 1)
            .single()
            .unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2024-02-01 23:59:00 UTC (first day of next month)
        let expected = Utc
            .with_ymd_and_hms(2024, 2, 1, 23, 59, 0)
            .single()
            .unwrap();

        assert_eq!(next_closing.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_year_boundary() {
        let broadcaster = TestBroadcaster::new("23:59:00", "UTC");

        // Current time: 2024-12-31 23:59:01 UTC (end of year, after closing)
        let now = Utc
            .with_ymd_and_hms(2024, 12, 31, 23, 59, 1)
            .single()
            .unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2025-01-01 23:59:00 UTC (first day of next year)
        let expected = Utc
            .with_ymd_and_hms(2025, 1, 1, 23, 59, 0)
            .single()
            .unwrap();

        assert_eq!(next_closing.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_closing_date_in_different_timezone() {
        // This test verifies the fix for the timezone date extraction issue
        let broadcaster = TestBroadcaster::new("01:00:00", "Asia/Tokyo");

        // Simulate: It's 16:00 UTC on Jan 15, which is 01:00 JST on Jan 16
        // The next closing should be calculated, and when we extract the date
        // in the configured timezone, it should be Jan 16, not Jan 15
        let now = Utc
            .with_ymd_and_hms(2024, 1, 15, 15, 59, 0)
            .single()
            .unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Next closing should be 2024-01-15 16:00:00 UTC (which is 2024-01-16 01:00:00 JST)
        let expected = Utc
            .with_ymd_and_hms(2024, 1, 15, 16, 0, 0)
            .single()
            .unwrap();

        assert_eq!(next_closing.with_timezone(&Utc), expected);

        // Now verify the date extraction in the configured timezone
        let closing_date = next_closing.date_naive();

        // The date should be Jan 16 (in JST), not Jan 15 (UTC)
        let expected_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();
        assert_eq!(closing_date, expected_date);

        // Verify the timezone is "Asia/Tokyo"
        assert_eq!(&broadcaster.config.daily.timezone, "Asia/Tokyo");
    }

    #[test]
    fn test_dst_spring_forward_gap() {
        // In US/Eastern, on March 10, 2024 at 2:00 AM, clocks spring forward to 3:00 AM
        // If closing time is set to 2:30 AM, that time doesn't exist
        let broadcaster = TestBroadcaster::new("02:30:00", "America/New_York");

        // Current time: March 10, 2024 at 1:00 AM EST (before DST transition)
        // This is 06:00 UTC
        let now = Utc.with_ymd_and_hms(2024, 3, 10, 6, 0, 0).single().unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // The closing time 2:30 AM doesn't exist, so it should resolve to a time after 3:00 AM EDT
        // We expect it to resolve to around 3:30 AM EDT which is 07:30 UTC
        let next_closing_local = next_closing.with_timezone(
            &broadcaster
                .config
                .daily
                .timezone
                .parse::<chrono_tz::Tz>()
                .unwrap(),
        );

        // Should be on March 10
        assert_eq!(next_closing_local.date_naive().day(), 10);
        // Should be after 3:00 AM (the post-DST time)
        assert!(next_closing_local.hour() >= 3);
    }

    #[test]
    fn test_dst_fall_back_ambiguous() {
        // In US/Eastern, on November 3, 2024 at 2:00 AM, clocks fall back to 1:00 AM
        // If closing time is set to 1:30 AM, that time occurs twice
        let broadcaster = TestBroadcaster::new("01:30:00", "America/New_York");

        // Current time: November 3, 2024 at 12:00 AM EST (before DST transition)
        // This is 04:00 UTC
        let now = Utc.with_ymd_and_hms(2024, 11, 3, 4, 0, 0).single().unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // The closing time 1:30 AM occurs twice (first in EDT, then in EST)
        // We should use the earlier occurrence (first 1:30 AM in EDT)
        // First 1:30 AM EDT is 05:30 UTC
        let expected = Utc
            .with_ymd_and_hms(2024, 11, 3, 5, 30, 0)
            .single()
            .unwrap();

        assert_eq!(next_closing.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_dst_transition_day_after() {
        // Test that after a DST transition day, the next day works normally
        let broadcaster = TestBroadcaster::new("02:30:00", "America/New_York");

        // Current time: March 11, 2024 at 1:00 AM EDT (day after spring forward)
        // This is 05:00 UTC
        let now = Utc.with_ymd_and_hms(2024, 3, 11, 5, 0, 0).single().unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Should calculate normally for March 11, 2:30 AM EDT = 06:30 UTC
        let expected = Utc
            .with_ymd_and_hms(2024, 3, 11, 6, 30, 0)
            .single()
            .unwrap();

        assert_eq!(next_closing.with_timezone(&Utc), expected);
    }
}
