mod helpers;

use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use uuid::Uuid;

use core_report::{
    CoreReportEvent, NewReportRun, ReportPublisher, ReportRun, ReportRunRepo, ReportRunState,
    ReportRunType,
};
use helpers::event;

async fn setup() -> anyhow::Result<(
    ReportRunRepo<event::DummyEvent>,
    obix::Outbox<event::DummyEvent>,
)> {
    let pool = helpers::init_pool().await?;
    let (clock, _time) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
    .await?;

    let publisher = ReportPublisher::new(&outbox);
    let report_runs = ReportRunRepo::new(&pool, &publisher);

    Ok((report_runs, outbox))
}

async fn create_test_report_run(
    report_runs: &ReportRunRepo<event::DummyEvent>,
    state: ReportRunState,
    run_type: ReportRunType,
) -> anyhow::Result<ReportRun> {
    let new_report_run = NewReportRun::builder()
        .external_id(format!("dagster-run-{}", Uuid::new_v4()))
        .state(state)
        .run_type(run_type)
        .build()
        .expect("all fields for new report run provided");

    let mut db = report_runs.begin_op().await?;
    let report_run = report_runs.create_in_op(&mut db, new_report_run).await?;
    db.commit().await?;

    Ok(report_run)
}

async fn update_report_run_state(
    report_runs: &ReportRunRepo<event::DummyEvent>,
    report_run_id: core_report::ReportRunId,
    state: ReportRunState,
    run_type: ReportRunType,
) -> anyhow::Result<ReportRun> {
    let mut db = report_runs.begin_op().await?;
    let mut report_run = report_runs.find_by_id_in_op(&mut db, report_run_id).await?;
    report_run.update_state(state, run_type, None);
    report_runs.update_in_op(&mut db, &mut report_run).await?;
    db.commit().await?;
    Ok(report_run)
}

/// `ReportRunCreated` is published when a new report run is synced from Dagster.
///
/// In practice this is triggered when the sync job processes a new run from Dagster's API.
///
/// This event is consumed by `admin-server` to notify subscribers of new report runs.
#[tokio::test]
async fn report_run_created_publishes_event() -> anyhow::Result<()> {
    let (report_runs, outbox) = setup().await?;

    let (report_run, recorded) = event::expect_event(
        &outbox,
        || async {
            create_test_report_run(&report_runs, ReportRunState::Queued, ReportRunType::Manual)
                .await
        },
        |result, e| match e {
            CoreReportEvent::ReportRunCreated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, report_run.id);
    assert_eq!(recorded.external_id, report_run.external_id);
    assert_eq!(recorded.state, ReportRunState::Queued);
    assert_eq!(recorded.run_type, ReportRunType::Manual);

    Ok(())
}

/// `ReportRunStateUpdated` is published when a report run's state changes.
///
/// In practice this is triggered when the sync job detects a state change from Dagster.
///
/// This event is consumed by `admin-server` to notify subscribers of report run updates.
#[tokio::test]
async fn report_run_state_updated_publishes_event() -> anyhow::Result<()> {
    let (report_runs, outbox) = setup().await?;

    let report_run = create_test_report_run(
        &report_runs,
        ReportRunState::Queued,
        ReportRunType::Scheduled,
    )
    .await?;
    let report_run_id = report_run.id;

    let (updated_report_run, recorded) = event::expect_event(
        &outbox,
        || async {
            update_report_run_state(
                &report_runs,
                report_run_id,
                ReportRunState::Running,
                ReportRunType::Scheduled,
            )
            .await
        },
        |result, e| match e {
            CoreReportEvent::ReportRunStateUpdated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, updated_report_run.id);
    assert_eq!(recorded.state, ReportRunState::Running);
    assert_eq!(recorded.run_type, ReportRunType::Scheduled);

    Ok(())
}
