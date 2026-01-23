mod helpers;

use authz::Authorization;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};

use core_access::{
    AuthRoleToken, CoreAccess, CoreAccessAction, CoreAccessEvent, CoreAccessObject,
    PermissionSetId, RoleId, config::AccessConfig,
};
use helpers::{TestAudit, TestSubject, event};

#[tokio::test]
async fn create_role_publishes_event() -> anyhow::Result<()> {
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

    // Add permission for role creation
    let test_role_id = RoleId::new();
    authz
        .add_permission_to_role(
            &test_role_id,
            &CoreAccessObject::all_roles(),
            &CoreAccessAction::ROLE_CREATE,
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

    let role_name = format!("test-role-{}", uuid::Uuid::new_v4());

    // Execute use case and wait for the expected event
    let (role, recorded) = event::expect_event(
        &outbox,
        || {
            access.create_role(
                &TestSubject,
                role_name.clone(),
                Vec::<PermissionSetId>::new(),
            )
        },
        |result, e| match e {
            CoreAccessEvent::RoleCreated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, role.id);
    assert_eq!(recorded.name, role_name);

    Ok(())
}
