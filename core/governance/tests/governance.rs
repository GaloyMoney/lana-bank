mod helpers;

use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use uuid::Uuid;

use governance::{
    ApprovalProcessId, ApprovalProcessStatus, ApprovalProcessType, Governance, GovernanceEvent,
};
use helpers::{action, event, object};
use obix::test_utils::expect_event;

/// Creates a test setup with all required dependencies for governance tests.
async fn setup() -> anyhow::Result<(
    Governance<
        authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>,
        event::DummyEvent,
    >,
    obix::Outbox<event::DummyEvent>,
    sqlx::PgPool,
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

    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let governance = Governance::new(&pool, &authz, &outbox, clock.clone());

    Ok((governance, outbox, pool))
}

/// ApprovalProcessConcluded event is published when a process concludes.
///
/// The event contains a snapshot including:
/// - id: The approval process id
/// - process_type: The process type identifier
/// - status: The final status (Approved or Denied)
/// - target_ref: The target reference for the process
#[tokio::test]
async fn approval_process_concluded_publishes_event() -> anyhow::Result<()> {
    let (governance, outbox, pool) = setup().await?;

    let process_type = ApprovalProcessType::new("test-approval-process");
    governance.init_policy(process_type.clone()).await?;

    let target_ref = format!("target-{}", Uuid::new_v4());
    let process_id = ApprovalProcessId::new();

    let (process, recorded) = expect_event(
        &outbox,
        || async {
            let mut db = es_entity::DbOp::init(&pool).await?;
            let process = governance
                .start_process(
                    &mut db,
                    process_id,
                    target_ref.clone(),
                    process_type.clone(),
                )
                .await?;
            db.commit().await?;
            Ok::<_, anyhow::Error>(process)
        },
        |result, event| match event {
            GovernanceEvent::ApprovalProcessConcluded { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, process.id);
    assert_eq!(recorded.process_type, process_type);
    assert_eq!(recorded.status, ApprovalProcessStatus::Approved);
    assert_eq!(recorded.target_ref, target_ref);

    Ok(())
}
