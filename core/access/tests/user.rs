mod helpers;

use std::time::Duration;

use authz::Authorization;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use tokio_stream::StreamExt;

use core_access::{
    AuthRoleToken, CoreAccess, CoreAccessAction, CoreAccessEvent, CoreAccessObject, PermissionSetId,
    RoleId, config::AccessConfig,
};
use helpers::{TestAudit, TestSubject, event};

#[tokio::test]
async fn create_user_publishes_event() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let (clock, _) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox =
        obix::Outbox::<event::DummyEvent>::init(&pool, obix::MailboxConfig::builder().build()?)
            .await?;

    let audit = TestAudit;
    let authz: Authorization<TestAudit, AuthRoleToken> = Authorization::init(&pool, &audit).await?;

    // Add all necessary permissions for TestSubject directly (bypass bootstrap)
    // This avoids loading existing policies that might have incompatible formats
    let test_role_id = RoleId::new();

    // Add permissions for role and user creation
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

    // Assign our test role to the test subject
    authz
        .assign_role_to_subject(TestSubject, test_role_id)
        .await?;

    // Initialize CoreAccess WITHOUT superuser bootstrap
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

    // Start listening for events
    let mut listener = outbox.listen_persisted(None);

    // Create the user
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());
    let user = access.create_user(&TestSubject, &email, role.id).await?;

    // Wait for the UserCreated event (skip other events like RoleCreated)
    let user_created_event = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let event = listener.next().await.expect("should receive an event");
            if let Some(CoreAccessEvent::UserCreated { entity }) =
                event.as_event::<CoreAccessEvent>()
            {
                return entity.clone();
            }
        }
    })
    .await?;

    // Verify the UserCreated event has the correct data
    assert_eq!(user_created_event.id, user.id);
    assert_eq!(user_created_event.email, email);
    assert_eq!(user_created_event.role_id, role.id);

    Ok(())
}
