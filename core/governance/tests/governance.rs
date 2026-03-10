mod helpers;

use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use uuid::Uuid;

use governance::{
    ApprovalProcessId, ApprovalProcessStatus, ApprovalProcessType, Governance, GovernanceEvent,
};
use helpers::{action, event, object};
use obix::test_utils::expect_event;

type TestAuthz = authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>;
type TestGovernance = Governance<TestAuthz, event::DummyEvent>;

/// Creates a test setup with all required dependencies for governance tests.
async fn setup() -> anyhow::Result<(
    TestGovernance,
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

    let authz = TestAuthz::new();
    let governance = Governance::new(&pool, &authz, &outbox, clock.clone(), None);

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
                .start_process_in_op(
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

/// When `RequireCommitteeApproval` is enabled and a default committee exists,
/// `init_policy` should create the policy with the default committee's rules
/// instead of `SystemAutoApprove`.
#[tokio::test]
#[serial_test::file_serial(governance_domain_config)]
async fn init_policy_uses_default_committee_when_require_committee_enabled() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let (clock, _time) = ClockHandle::artificial(ArtificialClockConfig::manual());
    let authz = TestAuthz::new();

    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
    .await?;

    // Initialize domain configs with RequireCommitteeApproval=true
    let startup_configs = vec![(
        "require-committee-approval".to_string(),
        serde_json::Value::Bool(true),
    )];
    let (_, _, exposed_readonly) = domain_config::init(
        &pool,
        &authz,
        domain_config::EncryptionConfig::default(),
        startup_configs,
    )
    .await?;

    let governance = Governance::new(
        &pool,
        &authz,
        &outbox,
        clock.clone(),
        Some(&exposed_readonly),
    );

    // Bootstrap default committee so init_policy can use it
    let member_id = governance::CommitteeMemberId::new();
    let committee = governance.bootstrap_default_committee(member_id).await?;

    let process_type = ApprovalProcessType::new("test-uses-default-committee");
    let policy = governance.init_policy(process_type).await?;
    assert_eq!(
        policy.rules,
        governance::ApprovalRules::Committee {
            committee_id: committee.id
        },
        "Policy should use default committee when RequireCommitteeApproval is enabled"
    );

    Ok(())
}

/// When `RequireCommitteeApproval` is enabled but no default committee has been
/// bootstrapped, `init_policy` should fail with `DefaultCommitteeNotFound`.
#[tokio::test]
#[serial_test::file_serial(governance_domain_config)]
async fn init_policy_fails_without_default_committee_when_require_committee_enabled()
-> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let (clock, _time) = ClockHandle::artificial(ArtificialClockConfig::manual());
    let authz = TestAuthz::new();

    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
    .await?;

    let startup_configs = vec![(
        "require-committee-approval".to_string(),
        serde_json::Value::Bool(true),
    )];
    let (_, _, exposed_readonly) = domain_config::init(
        &pool,
        &authz,
        domain_config::EncryptionConfig::default(),
        startup_configs,
    )
    .await?;

    let governance = Governance::new(
        &pool,
        &authz,
        &outbox,
        clock.clone(),
        Some(&exposed_readonly),
    );

    // Remove any "default" committee left over from other tests
    helpers::delete_default_committee(&pool).await?;

    // Do NOT bootstrap default committee
    let process_type = ApprovalProcessType::new("test-no-default-committee");
    let result = governance.init_policy(process_type).await;
    assert!(
        result.is_err(),
        "init_policy should fail when default committee is not bootstrapped"
    );
    assert!(
        matches!(
            result.err().unwrap(),
            governance::error::GovernanceError::DefaultCommitteeNotFound
        ),
        "Expected DefaultCommitteeNotFound error"
    );

    Ok(())
}

/// When `RequireCommitteeApproval` is enabled but a policy already exists
/// (created before the config was enabled), `init_policy` should return
/// the existing policy without error.
#[tokio::test]
#[serial_test::file_serial(governance_domain_config)]
async fn init_policy_returns_existing_policy_when_require_committee_enabled() -> anyhow::Result<()>
{
    let pool = helpers::init_pool().await?;
    let (clock, _time) = ClockHandle::artificial(ArtificialClockConfig::manual());
    let authz = TestAuthz::new();
    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
    .await?;

    // Step 1: Create policy WITHOUT RequireCommitteeApproval (config disabled)
    let governance_no_config = Governance::new(&pool, &authz, &outbox, clock.clone(), None);
    let process_type = ApprovalProcessType::new("test-existing-policy-before-config");
    let policy = governance_no_config
        .init_policy(process_type.clone())
        .await?;

    // Step 2: Enable RequireCommitteeApproval and recreate governance
    let startup_configs = vec![(
        "require-committee-approval".to_string(),
        serde_json::Value::Bool(true),
    )];
    let (_, _, exposed_readonly) = domain_config::init(
        &pool,
        &authz,
        domain_config::EncryptionConfig::default(),
        startup_configs,
    )
    .await?;
    let governance_with_config = Governance::new(
        &pool,
        &authz,
        &outbox,
        clock.clone(),
        Some(&exposed_readonly),
    );

    // Step 3: init_policy for the same process_type should return existing policy
    let existing = governance_with_config
        .init_policy(process_type.clone())
        .await?;
    assert_eq!(existing.id, policy.id);

    Ok(())
}

/// `ApprovalProcessConcluded` is published with `Approved` status when a
/// committee member approves and the approval threshold is met.
///
/// This test uses a committee-based policy (not auto-approve) so the process
/// stays `InProgress` after creation. A single committee member then approves,
/// meeting the threshold and triggering the `Concluded` event via the
/// post-persist publisher hook.
#[cfg(feature = "test-dummy")]
#[tokio::test]
async fn approval_process_concluded_on_approve_publishes_event() -> anyhow::Result<()> {
    let (governance, outbox, pool) = setup().await?;

    let member_id = governance::CommitteeMemberId::new();
    let committee = governance
        .create_committee(
            &authz::dummy::DummySubject,
            "approve-test-committee".to_string(),
            [member_id].into(),
        )
        .await?;

    let process_type = ApprovalProcessType::new("test-approve-concluded");
    let policy = governance.init_policy(process_type.clone()).await?;
    governance
        .assign_committee_to_policy(&authz::dummy::DummySubject, policy.id, committee.id)
        .await?;

    let target_ref = format!("approve-target-{}", uuid::Uuid::new_v4());
    let process_id = ApprovalProcessId::new();

    // Start process — committee-based, so it stays InProgress
    let mut db = es_entity::DbOp::init(&pool).await?;
    let process = governance
        .start_process_in_op(
            &mut db,
            process_id,
            target_ref.clone(),
            process_type.clone(),
        )
        .await?;
    db.commit().await?;
    assert_eq!(process.status(), ApprovalProcessStatus::InProgress);

    // Approve — meets threshold, should publish Concluded event
    let (_process, recorded) = expect_event(
        &outbox,
        || async {
            governance
                .test_approve_process(member_id, process_id)
                .await
                .map_err(anyhow::Error::from)
        },
        |_result, event| match event {
            GovernanceEvent::ApprovalProcessConcluded { entity } if entity.id == process_id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, process_id);
    assert_eq!(recorded.process_type, process_type);
    assert_eq!(recorded.status, ApprovalProcessStatus::Approved);
    assert_eq!(recorded.target_ref, target_ref);

    Ok(())
}

/// `ApprovalProcessConcluded` is published with `Denied` status when a
/// committee member denies a process.
///
/// Any single deny immediately concludes the process as denied. The test
/// verifies the `Concluded` event is published with `Denied` status and the
/// correct snapshot data.
#[cfg(feature = "test-dummy")]
#[tokio::test]
async fn approval_process_concluded_on_deny_publishes_event() -> anyhow::Result<()> {
    let (governance, outbox, pool) = setup().await?;

    let member_id = governance::CommitteeMemberId::new();
    let committee = governance
        .create_committee(
            &authz::dummy::DummySubject,
            "deny-test-committee".to_string(),
            [member_id].into(),
        )
        .await?;

    let process_type = ApprovalProcessType::new("test-deny-concluded");
    let policy = governance.init_policy(process_type.clone()).await?;
    governance
        .assign_committee_to_policy(&authz::dummy::DummySubject, policy.id, committee.id)
        .await?;

    let target_ref = format!("deny-target-{}", uuid::Uuid::new_v4());
    let process_id = ApprovalProcessId::new();

    // Start process — committee-based, so it stays InProgress
    let mut db = es_entity::DbOp::init(&pool).await?;
    let process = governance
        .start_process_in_op(
            &mut db,
            process_id,
            target_ref.clone(),
            process_type.clone(),
        )
        .await?;
    db.commit().await?;
    assert_eq!(process.status(), ApprovalProcessStatus::InProgress);

    // Deny — immediately concludes as denied, should publish Concluded event
    let (_process, recorded) = expect_event(
        &outbox,
        || async {
            governance
                .test_deny_process(member_id, process_id, "test denial reason".to_string())
                .await
                .map_err(anyhow::Error::from)
        },
        |_result, event| match event {
            GovernanceEvent::ApprovalProcessConcluded { entity } if entity.id == process_id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, process_id);
    assert_eq!(recorded.process_type, process_type);
    assert_eq!(recorded.status, ApprovalProcessStatus::Denied);
    assert_eq!(recorded.target_ref, target_ref);

    Ok(())
}

/// `bootstrap_default_committee` creates a committee with the given member
/// and is idempotent — calling it twice should not fail.
#[tokio::test]
#[serial_test::file_serial(governance_domain_config)]
async fn bootstrap_default_committee_is_idempotent() -> anyhow::Result<()> {
    let (governance, _outbox, _pool) = setup().await?;

    let member_id = governance::CommitteeMemberId::new();

    let committee1 = governance.bootstrap_default_committee(member_id).await?;
    let committee2 = governance.bootstrap_default_committee(member_id).await?;

    assert_eq!(committee1.id, committee2.id);
    assert_eq!(committee1.name, governance::DEFAULT_COMMITTEE_NAME);
    assert!(
        committee2.members().contains(&member_id),
        "Committee should contain the bootstrapped member"
    );

    Ok(())
}
