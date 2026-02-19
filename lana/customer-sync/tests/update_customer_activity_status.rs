mod helpers;

use authz::dummy::DummySubject;
use core_customer::{Activity, CoreCustomerEvent};
use core_time_events::CoreTimeEvent;

fn new_email() -> String {
    format!("test-{}@example.com", uuid::Uuid::new_v4())
}

fn new_telegram() -> String {
    format!("telegram-{}", uuid::Uuid::new_v4())
}

fn end_of_day_now() -> CoreTimeEvent {
    let now = chrono::Utc::now();
    CoreTimeEvent::EndOfDay {
        day: now.date_naive(),
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

    // ~2 years ago falls in the Inactive range (1–10 years)
    let two_years_ago = chrono::Utc::now() - chrono::Duration::days(730);
    ctx.customer_activity_repo
        .upsert_activity(customer.id, two_years_ago)
        .await?;

    let end_of_day_event = end_of_day_now();

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

    // ~15 years ago falls in the Suspended range (10+ years)
    let fifteen_years_ago = chrono::Utc::now() - chrono::Duration::days(5475);
    ctx.customer_activity_repo
        .upsert_activity(customer.id, fifteen_years_ago)
        .await?;

    let end_of_day_event = end_of_day_now();

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
    let two_years_ago = chrono::Utc::now() - chrono::Duration::days(730);
    ctx.customer_activity_repo
        .upsert_activity(customer.id, two_years_ago)
        .await?;
    ctx.customers
        .perform_customer_activity_status_update(chrono::Utc::now())
        .await?;

    // Now set recent activity and start the job poller
    let yesterday = chrono::Utc::now() - chrono::Duration::days(1);
    ctx.customer_activity_repo
        .upsert_activity(customer.id, yesterday)
        .await?;

    ctx.jobs.start_poll().await?;

    let end_of_day_event = end_of_day_now();

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
