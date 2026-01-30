mod helpers;

use chrono::Utc;
use dagster::graphql_client::{RunResult, RunStatus};

use core_report::{
    CoreReportEvent, ReportConfig, ReportPublisher, ReportRepo, ReportRunRepo, ReportRunState,
    ReportRunType, SyncReportsJobRunner,
};
use helpers::event;

async fn setup() -> anyhow::Result<(
    SyncReportsJobRunner<event::DummyEvent>,
    obix::Outbox<event::DummyEvent>,
)> {
    let pool = helpers::init_pool().await?;
    let outbox =
        obix::Outbox::<event::DummyEvent>::init(&pool, obix::MailboxConfig::builder().build()?)
            .await?;

    let publisher = ReportPublisher::new(&outbox);
    let report_runs = ReportRunRepo::new(&pool, &publisher);
    let reports = ReportRepo::new(&pool);
    let dagster = dagster::Dagster::new(ReportConfig::default().dagster);

    let runner = SyncReportsJobRunner::new(dagster, report_runs, reports);

    Ok((runner, outbox))
}

/// `ReportRunCreated` is published when a new report run is synced from Dagster
/// via `SyncReportsJobRunner::sync_run()`.
///
/// Current consumer:
/// - Admin-server GraphQL subscription `report_run_updated`.
#[tokio::test]
async fn publishes_report_run_created_event_on_new_run() -> anyhow::Result<()> {
    let (runner, outbox) = setup().await?;
    let run_id = job::JobId::new().to_string();

    let run_result = RunResult {
        run_id: run_id.clone(),
        status: RunStatus::Queued,
        start_time: Some(Utc::now()),
        tags: vec![],
    };

    let (_result, recorded) = event::expect_event(
        &outbox,
        || runner.sync_run(&run_result),
        |_, e| match e {
            CoreReportEvent::ReportRunCreated { entity }
                if entity.external_id == run_id && entity.state == ReportRunState::Queued =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(recorded.external_id, run_id);
    assert_eq!(recorded.state, ReportRunState::Queued);
    assert_eq!(recorded.run_type, ReportRunType::Manual);

    Ok(())
}

/// `ReportRunStateUpdated` is published when a report run's state changes
/// via `SyncReportsJobRunner::sync_run()`.
///
/// Current consumer:
/// - Admin-server GraphQL subscription `report_run_updated`.
#[tokio::test]
async fn publishes_report_run_state_updated_event_on_state_change() -> anyhow::Result<()> {
    let (runner, outbox) = setup().await?;
    let run_id = job::JobId::new().to_string();

    let queued = RunResult {
        run_id: run_id.clone(),
        status: RunStatus::Queued,
        start_time: Some(Utc::now()),
        tags: vec![],
    };
    runner
        .sync_run(&queued)
        .await
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    let started = RunResult {
        run_id: run_id.clone(),
        status: RunStatus::Started,
        start_time: Some(Utc::now()),
        tags: vec![],
    };

    let (_result, recorded) = event::expect_event(
        &outbox,
        || runner.sync_run(&started),
        |_, e| match e {
            CoreReportEvent::ReportRunStateUpdated { entity }
                if entity.external_id == run_id && entity.state == ReportRunState::Running =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(recorded.external_id, run_id);
    assert_eq!(recorded.state, ReportRunState::Running);
    assert_eq!(recorded.run_type, ReportRunType::Manual);

    Ok(())
}
