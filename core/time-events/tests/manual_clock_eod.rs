use anyhow::{Context, anyhow};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use sqlx::Row;
use tokio_stream::StreamExt;

use core_time_events::{CoreTimeEvent, TimeEvents};

const END_OF_DAY_PRODUCER_JOB_TYPE: &str = "cron.core-time-event.end-of-day-producer";

#[tokio::test]
#[serial_test::file_serial(core_time_events_manual_clock)]
async fn manual_clock_advance_publishes_end_of_day_event() -> anyhow::Result<()> {
    let pool = init_pool().await?;
    cleanup_end_of_day_job(&pool).await?;

    let start = Utc
        .with_ymd_and_hms(2024, 1, 1, 12, 0, 0)
        .single()
        .expect("valid timestamp");
    let (clock, controller) = ClockHandle::artificial(ArtificialClockConfig::manual_at(start));

    let outbox = obix::Outbox::<CoreTimeEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()
            .map_err(anyhow::Error::msg)?,
    )
    .await?;

    let authz = authz::dummy::DummyPerms::<DummyAction, DummyObject>::new();
    let (_, _, domain_configs) = domain_config::init(
        &pool,
        &authz,
        domain_config::EncryptionConfig::default(),
        vec![
            ("timezone".to_string(), serde_json::json!("UTC")),
            ("closing-time".to_string(), serde_json::json!("18:00:00")),
        ],
    )
    .await?;

    let mut jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .clock(clock.clone())
            .build()
            .map_err(anyhow::Error::msg)?,
    )
    .await?;

    let time_events = TimeEvents::init(
        &domain_configs,
        &mut jobs,
        &outbox,
        &clock,
        Some(controller),
    )
    .await?;

    jobs.start_poll().await?;

    let initial_state = time_events.state().await?;
    let expected_day = NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date");
    let expected_closing = Utc
        .with_ymd_and_hms(2024, 1, 1, 18, 0, 0)
        .single()
        .expect("valid timestamp");

    assert_eq!(initial_state.current_date, expected_day);
    assert_eq!(initial_state.current_time, start);
    assert_eq!(initial_state.next_end_of_day_at, expected_closing);
    assert!(initial_state.can_advance_to_next_end_of_day);

    wait_for_pending_schedule(&pool, expected_closing).await?;

    let mut listener = outbox.listen_persisted(None);

    let advanced_state = time_events.advance_to_next_end_of_day().await?;
    assert_eq!(advanced_state.current_time, expected_closing);
    assert_eq!(
        advanced_state.current_date, expected_day,
        "closing at the boundary should still report the business day that just closed",
    );

    let event = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let event = listener.next().await.expect("stream should not end");
            if let Some(CoreTimeEvent::EndOfDay {
                day,
                closing_time,
                timezone,
            }) = event.as_event::<CoreTimeEvent>()
            {
                return (*day, *closing_time, *timezone);
            }
        }
    })
    .await
    .context("timed out waiting for end-of-day event")?;

    assert_eq!(event.0, expected_day);
    assert_eq!(event.1, expected_closing);
    assert_eq!(event.2, chrono_tz::UTC);

    jobs.shutdown().await?;
    Ok(())
}

async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON").context("PG_CON must be set for integration tests")?;
    Ok(sqlx::PgPool::connect(&pg_con).await?)
}

async fn cleanup_end_of_day_job(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM job_executions WHERE job_type = $1")
        .bind(END_OF_DAY_PRODUCER_JOB_TYPE)
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        DELETE FROM job_events
        WHERE id IN (SELECT id FROM jobs WHERE job_type = $1)
        "#,
    )
    .bind(END_OF_DAY_PRODUCER_JOB_TYPE)
    .execute(pool)
    .await?;
    sqlx::query("DELETE FROM jobs WHERE job_type = $1")
        .bind(END_OF_DAY_PRODUCER_JOB_TYPE)
        .execute(pool)
        .await?;
    Ok(())
}

async fn wait_for_pending_schedule(
    pool: &sqlx::PgPool,
    expected_at: DateTime<Utc>,
) -> anyhow::Result<()> {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let row = sqlx::query(
                r#"
                SELECT execute_at
                FROM job_executions
                WHERE job_type = $1 AND state = 'pending'
                "#,
            )
            .bind(END_OF_DAY_PRODUCER_JOB_TYPE)
            .fetch_optional(pool)
            .await?;

            if let Some(row) = row {
                let execute_at: Option<DateTime<Utc>> = row.try_get("execute_at")?;
                if execute_at == Some(expected_at) {
                    return Ok::<(), anyhow::Error>(());
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    })
    .await
    .map_err(|_| anyhow!("timed out waiting for end-of-day producer to reschedule"))??;

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DummyAction;

impl From<domain_config::DomainConfigAction> for DummyAction {
    fn from(_: domain_config::DomainConfigAction) -> Self {
        Self
    }
}

impl std::fmt::Display for DummyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dummy")
    }
}

impl std::str::FromStr for DummyAction {
    type Err = strum::ParseError;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DummyObject;

impl From<domain_config::DomainConfigObject> for DummyObject {
    fn from(_: domain_config::DomainConfigObject) -> Self {
        Self
    }
}

impl std::fmt::Display for DummyObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dummy")
    }
}

impl std::str::FromStr for DummyObject {
    type Err = &'static str;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Self)
    }
}
