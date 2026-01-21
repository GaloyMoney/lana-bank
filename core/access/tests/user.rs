mod helpers;

use authz::Authorization;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};

use core_access::{
    AuthRoleToken, CoreAccess, CoreAccessAction, CoreAccessEvent, CoreAccessObject,
    PermissionSetId, RoleId, config::AccessConfig,
};
use helpers::{TestAudit, TestSubject, event};

#[tokio::test]
async fn create_user_publishes_event() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let (clock, _time) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
    .await?;

    let audit = TestAudit;
    let authz: Authorization<TestAudit, AuthRoleToken> = Authorization::init(&pool, &audit).await?;

    // Add all necessary permissions for TestSubject directly
    let test_role_id = RoleId::new();
    authz
        .add_permission_to_role(
            &test_role_id,
            &CoreAccessObject::all_roles(),
            &CoreAccessAction::ROLE_CREATE,
        )
        .await?;
    authz
        .add_permission_to_role(
            &test_role_id,
            &CoreAccessObject::all_users(),
            &CoreAccessAction::USER_CREATE,
        )
        .await?;
    authz
        .assign_role_to_subject(TestSubject, test_role_id)
        .await?;

    let config = AccessConfig {
        superuser_email: None,
    };

    let access = CoreAccess::init(
        &pool,
        config,
        CoreAccessAction::actions(),
        &[],
        &authz,
        &outbox,
        clock,
    )
    .await?;

    // Create a role first (needed for user creation)
    let role = access
        .create_role(
            &TestSubject,
            format!("test-role-{}", uuid::Uuid::new_v4()),
            Vec::<PermissionSetId>::new(),
        )
        .await?;

    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    // Execute use case and wait for the expected event
    let (user, recorded) = event::expect_event(
        &outbox,
        || access.create_user(&TestSubject, &email, role.id),
        |result, e| match e {
            CoreAccessEvent::UserCreated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, user.id);
    assert_eq!(recorded.email, email);
    assert_eq!(recorded.role_id, role.id);

    Ok(())
}
