mod helpers;

use serial_test::file_serial;

use lava_core::{
    audit::*,
    authorization::{error::AuthorizationError, *},
    primitives::*,
    user::{UserConfig, Users},
};
use uuid::Uuid;

fn random_email() -> String {
    format!("{}@integrationtest.com", Uuid::new_v4().to_string())
}

async fn init_users(
    pool: &sqlx::PgPool,
    authz: &Authorization,
    audit: &Audit,
) -> anyhow::Result<(Users, Subject)> {
    let superuser_email = "superuser@test.io";
    let users = Users::init(
        &pool,
        UserConfig {
            superuser_email: Some("superuser@test.io".to_string()),
        },
        &authz,
        &audit,
    )
    .await?;
    let superuser = users
        .find_by_email(superuser_email)
        .await?
        .expect("Superuser not found");
    Ok((users, Subject::from(superuser.id)))
}

async fn create_user_with_role(
    users: &Users,
    superuser_subject: &Subject,
    role: Role,
) -> anyhow::Result<Subject> {
    let user = users
        .create_user(&superuser_subject, random_email())
        .await?;
    let user = users
        .assign_role_to_user(&superuser_subject, user.id, role)
        .await?;
    Ok(Subject::from(user.id))
}

#[tokio::test]
#[file_serial]
async fn superuser_permissions() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let (_, superuser_subject) = init_users(&pool, &authz, &audit).await?;

    // Superuser can create users
    assert!(authz
        .check_permission(
            &superuser_subject,
            Object::User,
            Action::User(UserAction::Create),
        )
        .await
        .is_ok());

    // Superuser can assign Admin role
    assert!(authz
        .check_permission(
            &superuser_subject,
            Object::User,
            Action::User(UserAction::AssignRole(Role::Admin))
        )
        .await
        .is_ok());

    // Superuser can assign Bank Manager role
    assert!(authz
        .check_permission(
            &superuser_subject,
            Object::User,
            Action::User(UserAction::AssignRole(Role::BankManager))
        )
        .await
        .is_ok());

    // Superuser cannot assign Superuser role
    assert!(matches!(
        authz
            .check_permission(
                &superuser_subject,
                Object::User,
                Action::User(UserAction::AssignRole(Role::Superuser))
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn admin_permissions() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let (users, superuser_subject) = init_users(&pool, &authz, &audit).await?;

    let admin_subject = create_user_with_role(&users, &superuser_subject, Role::Admin).await?;

    // Admin can create users
    assert!(authz
        .check_permission(
            &admin_subject,
            Object::User,
            Action::User(UserAction::Create)
        )
        .await
        .is_ok());

    // Admin can assign Bank Manager role
    assert!(authz
        .check_permission(
            &admin_subject,
            Object::User,
            Action::User(UserAction::AssignRole(Role::BankManager))
        )
        .await
        .is_ok());

    // Admin cannot assign Admin role
    assert!(matches!(
        authz
            .check_permission(
                &admin_subject,
                Object::User,
                Action::User(UserAction::AssignRole(Role::Admin))
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    // Admin cannot assign Superuser role
    assert!(matches!(
        authz
            .check_permission(
                &admin_subject,
                Object::User,
                Action::User(UserAction::AssignRole(Role::Superuser))
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn bank_manager_permissions() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let (users, superuser_subject) = init_users(&pool, &authz, &audit).await?;

    let bank_manager_subject =
        create_user_with_role(&users, &superuser_subject, Role::BankManager).await?;

    // Bank Manager cannot create users
    assert!(matches!(
        authz
            .check_permission(
                &bank_manager_subject,
                Object::User,
                Action::User(UserAction::Create)
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    // Bank Manager cannot assign roles
    assert!(matches!(
        authz
            .check_permission(
                &bank_manager_subject,
                Object::User,
                Action::User(UserAction::AssignRole(Role::BankManager))
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    assert!(matches!(
        authz
            .check_permission(
                &bank_manager_subject,
                Object::User,
                Action::User(UserAction::AssignRole(Role::Admin))
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    assert!(matches!(
        authz
            .check_permission(
                &bank_manager_subject,
                Object::User,
                Action::User(UserAction::AssignRole(Role::Superuser))
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    Ok(())
}
