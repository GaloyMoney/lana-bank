mod helpers;

use authz::dummy::DummySubject;
use core_credit::*;
use core_time_events::CoreTimeEvent;
use es_entity::DbOp;
use helpers::event::{DummyEvent, expect_event};

async fn publish_end_of_day(
    outbox: &obix::Outbox<DummyEvent>,
    pool: &sqlx::PgPool,
    clock: &es_entity::clock::ClockHandle,
    day: chrono::NaiveDate,
) -> anyhow::Result<()> {
    let mut op = DbOp::init_with_clock(pool, clock).await?;
    outbox
        .publish_persisted_in_op(
            &mut op,
            CoreTimeEvent::EndOfDay {
                day,
                closing_time: chrono::Utc::now(),
                timezone: chrono_tz::UTC,
            },
        )
        .await?;
    op.commit().await?;
    Ok(())
}

/// Credit facility transitions to `Matured` when an EndOfDay event is published
/// for its maturity date.
///
/// # Pipeline
/// ```text
/// EndOfDay(maturity_date)
///   → CreditFacilityMaturityEndOfDayHandler
///     → ProcessFacilityMaturitiesJob (batch query)
///       → CreditFacilityMaturityJob (per-facility)
///         → facility.mature()
/// ```
///
/// # Verified
/// - `FacilityMatured` event is published with the correct facility id
async fn cleanup_stale_task_jobs(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    sqlx::query(
        "DELETE FROM job_executions
         WHERE state = 'pending'
           AND job_type IN ('task.process-facility-maturities', 'task.credit-facility-maturity')",
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn facility_matures_on_end_of_day() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    cleanup_stale_task_jobs(&ctx.pool).await?;
    ctx.jobs.start_poll().await?;

    let state = helpers::create_active_facility(&ctx, helpers::test_terms()).await?;

    let facility = ctx
        .credit
        .facilities()
        .find_by_id(&DummySubject, state.facility_id)
        .await?
        .expect("facility must exist");
    assert_eq!(facility.status(), CreditFacilityStatus::Active);

    let maturity_date = facility.maturity_date();
    let facility_id = state.facility_id;

    let outbox = ctx.outbox.clone();
    let pool = ctx.pool.clone();
    let clock = ctx.clock.clone();

    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || async move {
            publish_end_of_day(&outbox, &pool, &clock, maturity_date).await?;
            Ok::<_, anyhow::Error>(())
        },
        move |_, e| match e {
            DummyEvent::CoreCredit(CoreCreditEvent::FacilityMatured { entity })
                if entity.id == facility_id =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, facility_id);
    assert_eq!(recorded.customer_id, state.customer_id);
    assert_eq!(recorded.amount, state.amount);

    ctx.jobs.shutdown().await?;
    cleanup_stale_task_jobs(&ctx.pool).await?;
    Ok(())
}
