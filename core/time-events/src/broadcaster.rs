use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use tracing::{error, info, instrument};

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
            .map_err(|_| TimeEventsError::InvalidTimezone(self.config.daily.timezone.clone()))
    }

    fn parse_closing_time(&self) -> Result<NaiveTime, TimeEventsError> {
        NaiveTime::parse_from_str(&self.config.daily.closing_time, "%H:%M:%S")
            .map_err(|_| TimeEventsError::InvalidTimeFormat(self.config.daily.closing_time.clone()))
    }

    fn calculate_next_closing(&self, now: DateTime<Utc>) -> Result<DateTime<Utc>, TimeEventsError> {
        let tz = self.parse_timezone()?;
        let closing_time = self.parse_closing_time()?;

        let now_in_tz = now.with_timezone(&tz);
        let today_closing = tz
            .from_local_datetime(&now_in_tz.date_naive().and_time(closing_time))
            .single()
            .ok_or_else(|| {
                TimeEventsError::InvalidTimeFormat(format!(
                    "Could not create datetime for closing time: {}",
                    closing_time
                ))
            })?;

        let next_closing = if now_in_tz < today_closing {
            today_closing
        } else {
            let tomorrow = now_in_tz.date_naive() + chrono::Duration::days(1);
            tz.from_local_datetime(&tomorrow.and_time(closing_time))
                .single()
                .ok_or_else(|| {
                    TimeEventsError::InvalidTimeFormat(format!(
                        "Could not create datetime for closing time: {}",
                        closing_time
                    ))
                })?
        };

        Ok(next_closing.with_timezone(&Utc))
    }

    #[instrument(
        name = "time_events.broadcaster.publish_daily_closing",
        skip(self),
        fields(date = %date, timezone = %self.config.daily.timezone),
        err
    )]
    async fn publish_daily_closing(&self, date: chrono::NaiveDate) -> Result<(), TimeEventsError> {
        self.outbox
            .publish_ephemeral(
                EphemeralEventType::new("time.daily-closing"),
                TimeEvent::DailyClosing { date },
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

        // Parse timezone once at startup - it won't change during runtime
        let tz = match self.parse_timezone() {
            Ok(tz) => tz,
            Err(e) => {
                error!(error = %e, "Failed to parse timezone, broadcaster cannot start");
                return;
            }
        };

        loop {
            let now = Utc::now();

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

                        // Convert to configured timezone to get the correct date
                        let closing_date = next_closing.with_timezone(&tz).date_naive();

                        loop {
                            match self.publish_daily_closing(closing_date).await {
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
                .map_err(|_| TimeEventsError::InvalidTimezone(self.config.daily.timezone.clone()))
        }

        fn parse_closing_time(&self) -> Result<NaiveTime, TimeEventsError> {
            NaiveTime::parse_from_str(&self.config.daily.closing_time, "%H:%M:%S").map_err(|_| {
                TimeEventsError::InvalidTimeFormat(self.config.daily.closing_time.clone())
            })
        }

        fn calculate_next_closing(
            &self,
            now: DateTime<Utc>,
        ) -> Result<DateTime<Utc>, TimeEventsError> {
            let tz = self.parse_timezone()?;
            let closing_time = self.parse_closing_time()?;

            let now_in_tz = now.with_timezone(&tz);
            let today_closing = tz
                .from_local_datetime(&now_in_tz.date_naive().and_time(closing_time))
                .single()
                .ok_or_else(|| {
                    TimeEventsError::InvalidTimeFormat(format!(
                        "Could not create datetime for closing time: {}",
                        closing_time
                    ))
                })?;

            let next_closing = if now_in_tz < today_closing {
                today_closing
            } else {
                let tomorrow = now_in_tz.date_naive() + chrono::Duration::days(1);
                tz.from_local_datetime(&tomorrow.and_time(closing_time))
                    .single()
                    .ok_or_else(|| {
                        TimeEventsError::InvalidTimeFormat(format!(
                            "Could not create datetime for closing time: {}",
                            closing_time
                        ))
                    })?
            };

            Ok(next_closing.with_timezone(&Utc))
        }
    }

    #[test]
    fn test_next_closing_before_closing_time_utc() {
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

        assert_eq!(next_closing, expected);
    }

    #[test]
    fn test_next_closing_after_closing_time_utc() {
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

        assert_eq!(next_closing, expected);
    }

    #[test]
    fn test_next_closing_exactly_at_closing_time() {
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

        assert_eq!(next_closing, expected);
    }

    #[test]
    fn test_next_closing_different_timezone_america_new_york() {
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

        assert_eq!(next_closing, expected);
    }

    #[test]
    fn test_next_closing_different_timezone_asia_tokyo() {
        let broadcaster = TestBroadcaster::new("18:00:00", "Asia/Tokyo");

        // Current time: 2024-01-15 08:00:00 UTC (17:00 JST, before 18:00 JST closing)
        let now = Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).single().unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2024-01-15 09:00:00 UTC (18:00 JST = 09:00 UTC)
        let expected = Utc.with_ymd_and_hms(2024, 1, 15, 9, 0, 0).single().unwrap();

        assert_eq!(next_closing, expected);
    }

    #[test]
    fn test_next_closing_midnight_closing() {
        let broadcaster = TestBroadcaster::new("00:00:00", "UTC");

        // Current time: 2024-01-15 23:00:00 UTC (before midnight)
        let now = Utc
            .with_ymd_and_hms(2024, 1, 15, 23, 0, 0)
            .single()
            .unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2024-01-16 00:00:00 UTC (next midnight)
        let expected = Utc.with_ymd_and_hms(2024, 1, 16, 0, 0, 0).single().unwrap();

        assert_eq!(next_closing, expected);
    }

    #[test]
    fn test_next_closing_after_midnight() {
        let broadcaster = TestBroadcaster::new("00:00:00", "UTC");

        // Current time: 2024-01-16 00:00:01 UTC (just after midnight)
        let now = Utc.with_ymd_and_hms(2024, 1, 16, 0, 0, 1).single().unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2024-01-17 00:00:00 UTC (next midnight)
        let expected = Utc.with_ymd_and_hms(2024, 1, 17, 0, 0, 0).single().unwrap();

        assert_eq!(next_closing, expected);
    }

    #[test]
    fn test_next_closing_noon() {
        let broadcaster = TestBroadcaster::new("12:00:00", "UTC");

        // Current time: 2024-01-15 11:30:00 UTC (before noon)
        let now = Utc
            .with_ymd_and_hms(2024, 1, 15, 11, 30, 0)
            .single()
            .unwrap();

        let next_closing = broadcaster.calculate_next_closing(now).unwrap();

        // Expected: 2024-01-15 12:00:00 UTC (today at noon)
        let expected = Utc
            .with_ymd_and_hms(2024, 1, 15, 12, 0, 0)
            .single()
            .unwrap();

        assert_eq!(next_closing, expected);
    }

    #[test]
    fn test_invalid_timezone() {
        let broadcaster = TestBroadcaster::new("12:00:00", "Invalid/Timezone");

        let now = Utc::now();
        let result = broadcaster.calculate_next_closing(now);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TimeEventsError::InvalidTimezone(_)
        ));
    }

    #[test]
    fn test_invalid_time_format() {
        let broadcaster = TestBroadcaster::new("25:00:00", "UTC");

        let now = Utc::now();
        let result = broadcaster.calculate_next_closing(now);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TimeEventsError::InvalidTimeFormat(_)
        ));
    }

    #[test]
    fn test_invalid_time_format_wrong_pattern() {
        let broadcaster = TestBroadcaster::new("12:00", "UTC");

        let now = Utc::now();
        let result = broadcaster.calculate_next_closing(now);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TimeEventsError::InvalidTimeFormat(_)
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

        assert_eq!(next_closing, expected);
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

        assert_eq!(next_closing, expected);
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

        assert_eq!(next_closing, expected);

        // Now verify the date extraction in the configured timezone
        let tz = broadcaster.parse_timezone().unwrap();
        let closing_date = next_closing.with_timezone(&tz).date_naive();

        // The date should be Jan 16 (in JST), not Jan 15 (UTC)
        let expected_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();
        assert_eq!(closing_date, expected_date);

        // Verify the timezone is "Asia/Tokyo"
        assert_eq!(&broadcaster.config.daily.timezone, "Asia/Tokyo");
    }
}
