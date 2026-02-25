mod helpers;

use authz::dummy::DummySubject;
use core_credit::*;
use core_time_events::CoreTimeEvent;
use es_entity::DbOp;
use helpers::event::DummyEvent;
use std::time::Duration;

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
/// - Facility status transitions from `Active` to `Matured`
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn facility_matures_on_end_of_day() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
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

    publish_end_of_day(&ctx.outbox, &ctx.pool, &ctx.clock, maturity_date).await?;

    for attempt in 0..100 {
        let facility = ctx
            .credit
            .facilities()
            .find_by_id(&DummySubject, state.facility_id)
            .await?
            .expect("facility must exist");
        if facility.status() == CreditFacilityStatus::Matured {
            break;
        }
        if attempt == 99 {
            panic!("Timed out waiting for facility to mature after 10 seconds");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    ctx.jobs.shutdown().await?;
    Ok(())
}
