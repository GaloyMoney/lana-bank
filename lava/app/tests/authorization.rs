mod helpers;

use serial_test::file_serial;

use authz::PermissionCheck;

use lava_app::{
    audit::*,
    authorization::{error::AuthorizationError, init as init_authz, *},
    outbox::Outbox,
    primitives::*,
    user::Users,
};
use uuid::Uuid;

fn random_email() -> String {
    format!("{}@integrationtest.com", Uuid::new_v4())
}

async fn init_users(
    pool: &sqlx::PgPool,
    authz: &Authorization,
) -> anyhow::Result<(Users, Subject)> {
    let superuser_email = "superuser@test.io".to_string();
    let outbox = Outbox::init(&pool).await?;
    let users = Users::init(&pool, &authz, &outbox, Some(superuser_email.clone())).await?;
    let superuser = users
        .find_by_email(&superuser_email)
        .await?
        .expect("Superuser not found");
    Ok((users, Subject::from(superuser.id)))
}

async fn create_user_with_role(
    users: &Users,
    superuser_subject: &Subject,
    role: Role,
) -> anyhow::Result<Subject> {
    let user = users.create_user(superuser_subject, random_email()).await?;
    let user = users
        .assign_role_to_user(superuser_subject, user.id, role)
        .await?;
    Ok(Subject::from(user.id))
}

#[tokio::test]
#[file_serial]
async fn superuser_permissions() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = init_authz(&pool, &audit).await?;
    let (_, superuser_subject) = init_users(&pool, &authz).await?;

    // Superuser can create users
    assert!(authz
        .enforce_permission(
            &superuser_subject,
            UserObject::all_users(),
            UserModuleAction::USER_CREATE,
        )
        .await
        .is_ok());

    // Superuser can assign Admin role
    assert!(authz
        .enforce_permission(
            &superuser_subject,
            UserObject::all_users(),
            UserModuleAction::USER_ASSIGN_ROLE,
        )
        .await
        .is_ok());

    // Superuser can assign Bank Manager role
    assert!(authz
        .enforce_permission(
            &superuser_subject,
            UserObject::user(UserId::new()),
            UserModuleAction::USER_ASSIGN_ROLE,
        )
        .await
        .is_ok());

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn admin_permissions() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = init_authz(&pool, &audit).await?;
    let (users, superuser_subject) = init_users(&pool, &authz).await?;

    let admin_subject = create_user_with_role(&users, &superuser_subject, LavaRole::ADMIN).await?;

    // Admin can create users
    assert!(authz
        .enforce_permission(
            &admin_subject,
            UserObject::all_users(),
            UserModuleAction::USER_CREATE,
        )
        .await
        .is_ok());

    // Admin can assign roles
    assert!(authz
        .enforce_permission(
            &admin_subject,
            UserObject::all_users(),
            UserModuleAction::USER_ASSIGN_ROLE,
        )
        .await
        .is_ok());
    assert!(authz
        .enforce_permission(
            &admin_subject,
            UserObject::user(UserId::new()),
            UserModuleAction::USER_ASSIGN_ROLE,
        )
        .await
        .is_ok());

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn bank_manager_permissions() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = init_authz(&pool, &audit).await?;
    let (users, superuser_subject) = init_users(&pool, &authz).await?;

    let bank_manager_subject =
        create_user_with_role(&users, &superuser_subject, LavaRole::BANK_MANAGER).await?;

    // Bank Manager cannot create users
    assert!(matches!(
        authz
            .enforce_permission(
                &bank_manager_subject,
                UserObject::all_users(),
                UserModuleAction::USER_CREATE,
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    // Bank Manager cannot assign roles
    assert!(matches!(
        authz
            .enforce_permission(
                &bank_manager_subject,
                UserObject::all_users(),
                UserModuleAction::USER_ASSIGN_ROLE,
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    Ok(())
}
