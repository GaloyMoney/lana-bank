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

/// `ApprovalProcessConcluded` is published when an approval process reaches a terminal state.
///
/// This event is consumed by `core_deposit` to finalize withdrawal approvals and by `core_credit` to finalize disbursal approvals and credit facility proposal approvals.
///
/// This test uses `Governance::init_policy()` which defaults to system auto-approval, so the process concludes immediately when started.
///
/// The event contains a snapshot including the process id, process type, final status, and target reference.
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
