mod helpers;

use authz::dummy::DummySubject;
use chrono::{NaiveDate, TimeZone};
use core_customer::{Activity, CoreCustomerEvent};
use core_time_events::CoreTimeEvent;

fn new_email() -> String {
    format!("test-{}@example.com", uuid::Uuid::new_v4())
}

fn new_telegram() -> String {
    format!("telegram-{}", uuid::Uuid::new_v4())
}

/// Fixed "today" used across all tests: 2025-01-15T12:00:00Z
fn today() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap()
}

fn end_of_day_event() -> CoreTimeEvent {
    let now = today();
    CoreTimeEvent::EndOfDay {
        day: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        closing_time: now,
        timezone: chrono_tz::Tz::UTC,
    }
}

/// Activity date 1–10 years ago → handler transitions Active → Inactive
#[tokio::test]
async fn end_of_day_transitions_active_to_inactive() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let customer = ctx
        .customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            new_email(),
            new_telegram(),
            core_customer::CustomerType::Individual,
        )
        .await?;
    assert_eq!(customer.activity, Activity::Active);

    // 2023-01-15 is ~2 years before "today" (2025-01-15), falls in Inactive range (1–10 years)
    let two_years_ago = chrono::Utc.with_ymd_and_hms(2023, 1, 15, 12, 0, 0).unwrap();
    ctx.customer_activity_repo
        .upsert_activity(customer.id, two_years_ago)
        .await?;

    let end_of_day_event = end_of_day_event();

    let entity = helpers::expect_handler_reaction(
        &ctx.outbox,
        end_of_day_event,
        |_result, event: &CoreCustomerEvent| match event {
            CoreCustomerEvent::CustomerActivityUpdated { entity } if entity.id == customer.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(entity.id, customer.id);

    ctx.jobs.shutdown().await?;
    Ok(())
}

/// Activity date 10+ years ago → handler transitions Active → Suspended
#[tokio::test]
async fn end_of_day_transitions_active_to_suspended() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let customer = ctx
        .customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            new_email(),
            new_telegram(),
            core_customer::CustomerType::Individual,
        )
        .await?;
    assert_eq!(customer.activity, Activity::Active);

    // 2010-01-15 is ~15 years before "today" (2025-01-15), falls in Suspended range (10+ years)
    let fifteen_years_ago = chrono::Utc.with_ymd_and_hms(2010, 1, 15, 12, 0, 0).unwrap();
    ctx.customer_activity_repo
        .upsert_activity(customer.id, fifteen_years_ago)
        .await?;

    let end_of_day_event = end_of_day_event();

    let entity = helpers::expect_handler_reaction(
        &ctx.outbox,
        end_of_day_event,
        |_result, event: &CoreCustomerEvent| match event {
            CoreCustomerEvent::CustomerActivityUpdated { entity } if entity.id == customer.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(entity.id, customer.id);

    ctx.jobs.shutdown().await?;
    Ok(())
}

/// Customer already Inactive with recent activity → handler transitions Inactive → Active
#[tokio::test]
async fn end_of_day_transitions_inactive_to_active() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;

    let customer = ctx
        .customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            new_email(),
            new_telegram(),
            core_customer::CustomerType::Individual,
        )
        .await?;

    // First transition the customer to Inactive by giving them old activity
    // and calling perform_customer_activity_status_update directly.
    let two_years_ago = chrono::Utc.with_ymd_and_hms(2023, 1, 15, 12, 0, 0).unwrap();
    ctx.customer_activity_repo
        .upsert_activity(customer.id, two_years_ago)
        .await?;
    ctx.customers
        .perform_customer_activity_status_update(today())
        .await?;

    // Now set recent activity (2025-01-14, one day before "today") and start the job poller
    let yesterday = chrono::Utc.with_ymd_and_hms(2025, 1, 14, 12, 0, 0).unwrap();
    ctx.customer_activity_repo
        .upsert_activity(customer.id, yesterday)
        .await?;

    ctx.jobs.start_poll().await?;

    let end_of_day_event = end_of_day_event();

    let entity = helpers::expect_handler_reaction(
        &ctx.outbox,
        end_of_day_event,
        |_result, event: &CoreCustomerEvent| match event {
            CoreCustomerEvent::CustomerActivityUpdated { entity } if entity.id == customer.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(entity.id, customer.id);

    ctx.jobs.shutdown().await?;
    Ok(())
}
