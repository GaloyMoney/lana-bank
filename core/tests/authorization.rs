mod helpers;

use serial_test::file_serial;

use lava_core::{
    audit::*,
    authorization::{error::AuthorizationError, *},
    primitives::*,
    user::Users,
};
use uuid::Uuid;

fn random_email() -> String {
    format!("{}@integrationtest.com", Uuid::new_v4())
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
    let authz = Authorization::init(&pool, &audit).await?;
    let (_, _, superuser_subject) = helpers::init_users(&pool, &authz, &audit).await?;

    // Superuser can create users
    assert!(authz
        .enforce_permission(
            &superuser_subject,
            Object::User,
            Action::User(UserAction::Create)
        )
        .await
        .is_ok());

    // Superuser can assign Admin role
    assert!(authz
        .enforce_permission(
            &superuser_subject,
            Object::User,
            Action::User(UserAction::AssignRole)
        )
        .await
        .is_ok());

    // Superuser can assign Bank Manager role
    assert!(authz
        .enforce_permission(
            &superuser_subject,
            Object::User,
            Action::User(UserAction::AssignRole)
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
    let authz = Authorization::init(&pool, &audit).await?;
    let (users, _, superuser_subject) = helpers::init_users(&pool, &authz, &audit).await?;

    let admin_subject = create_user_with_role(&users, &superuser_subject, Role::Admin).await?;

    // Admin can create users
    assert!(authz
        .enforce_permission(
            &admin_subject,
            Object::User,
            Action::User(UserAction::Create)
        )
        .await
        .is_ok());

    // Admin can assign Bank Manager role
    assert!(authz
        .enforce_permission(
            &admin_subject,
            Object::User,
            Action::User(UserAction::AssignRole)
        )
        .await
        .is_ok());

    // Admin can assign roles
    assert!(authz
        .enforce_permission(
            &admin_subject,
            Object::User,
            Action::User(UserAction::AssignRole)
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
    let authz = Authorization::init(&pool, &audit).await?;
    let (users, _, superuser_subject) = helpers::init_users(&pool, &authz, &audit).await?;

    let bank_manager_subject =
        create_user_with_role(&users, &superuser_subject, Role::BankManager).await?;

    // Bank Manager cannot create users
    assert!(matches!(
        authz
            .enforce_permission(
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
            .enforce_permission(
                &bank_manager_subject,
                Object::User,
                Action::User(UserAction::AssignRole)
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    Ok(())
}
