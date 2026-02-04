mod helpers;

use authz::Authorization;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use rand::distr::{Alphanumeric, SampleString};
use serial_test::file_serial;

use core_access::{
    AuthRoleToken, CoreAccess, CoreAccessAction, CoreAccessEvent, CoreAccessObject,
    PermissionSetId, RoleId, config::AccessConfig,
};
use helpers::{
    ROLE_NAME_ACCOUNTANT, ROLE_NAME_ADMIN, ROLE_NAME_BANK_MANAGER, TestAudit, TestSubject, event,
};

fn generate_random_email() -> String {
    let random_string: String = Alphanumeric.sample_string(&mut rand::rng(), 32);
    format!("{}@example.com", random_string.to_lowercase())
}

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

    let subject = TestSubject::new();

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
        .assign_role_to_subject(subject, test_role_id)
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
            &subject,
            format!("test-role-{}", uuid::Uuid::new_v4()),
            Vec::<PermissionSetId>::new(),
        )
        .await?;

    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    // Execute use case and wait for the expected event
    let (user, recorded) = event::expect_event(
        &outbox,
        || access.create_user(&subject, &email, role.id),
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

#[tokio::test]
#[file_serial]
async fn user_lifecycle() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let (access, superuser_subject) = helpers::init_access(&pool).await?;

    let user_email = generate_random_email();

    let bank_manager_role = access
        .find_role_by_name(&superuser_subject, ROLE_NAME_BANK_MANAGER)
        .await?;

    let user = access
        .create_user(&superuser_subject, user_email.clone(), bank_manager_role.id)
        .await?;

    assert_eq!(user.email, user_email);
    assert_eq!(user.current_role(), bank_manager_role.id);

    // Test updating user role to admin
    let admin_role = access
        .find_role_by_name(&superuser_subject, ROLE_NAME_ADMIN)
        .await?;

    let updated_user = access
        .update_role_of_user(&superuser_subject, user.id, admin_role.id)
        .await?;

    assert_eq!(updated_user.id, user.id);
    assert_eq!(updated_user.email, user_email);
    assert_eq!(updated_user.current_role(), admin_role.id);

    // Test updating user role to accountant
    let accountant_role = access
        .find_role_by_name(&superuser_subject, ROLE_NAME_ACCOUNTANT)
        .await?;

    let final_user = access
        .update_role_of_user(&superuser_subject, user.id, accountant_role.id)
        .await?;

    assert_eq!(final_user.id, user.id);
    assert_eq!(final_user.email, user_email);
    assert_eq!(final_user.current_role(), accountant_role.id);

    Ok(())
}
