mod helpers;

use core_access::ROLE_NAME_SUPERUSER;
use serial_test::file_serial;

use authz::PermissionCheck;

use lana_app::{
    access::Access,
    audit::*,
    authorization::{error::AuthorizationError, init as init_authz, *},
    primitives::*,
};
use uuid::Uuid;

fn random_email() -> String {
    format!("{}@integrationtest.com", Uuid::new_v4())
}

async fn create_user_with_role(
    access: &Access,
    superuser_subject: &Subject,
    role_id: RoleId,
) -> anyhow::Result<Subject> {
    let user = access
        .users()
        .create_user(superuser_subject, random_email())
        .await?;
    let user = access
        .assign_role_to_user(superuser_subject, user.id, role_id)
        .await?;
    Ok(Subject::from(user.id))
}

#[tokio::test]
#[file_serial]
async fn superuser_permissions() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = init_authz(&pool, &audit).await?;
    let (_, superuser_subject) = helpers::init_access(&pool, &authz).await?;

    // Superuser can create users
    assert!(authz
        .enforce_permission(
            &superuser_subject,
            CoreAccessObject::all_users(),
            CoreAccessAction::USER_CREATE,
        )
        .await
        .is_ok());

    // Superuser can assign Admin role
    assert!(authz
        .enforce_permission(
            &superuser_subject,
            CoreAccessObject::all_users(),
            CoreAccessAction::USER_ASSIGN_ROLE,
        )
        .await
        .is_ok());

    // Superuser can assign Bank Manager role
    assert!(authz
        .enforce_permission(
            &superuser_subject,
            CoreAccessObject::user(UserId::new()),
            CoreAccessAction::USER_ASSIGN_ROLE,
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
    let (access, superuser_subject) = helpers::init_access(&pool, &authz).await?;

    let admin_role = access
        .find_role_by_name(&superuser_subject, ROLE_NAME_SUPERUSER)
        .await?;

    let admin_subject = create_user_with_role(&access, &superuser_subject, admin_role.id).await?;

    // Admin can create users
    assert!(authz
        .enforce_permission(
            &admin_subject,
            CoreAccessObject::all_users(),
            CoreAccessAction::USER_CREATE,
        )
        .await
        .is_ok());

    // Admin can assign roles
    assert!(authz
        .enforce_permission(
            &admin_subject,
            CoreAccessObject::all_users(),
            CoreAccessAction::USER_ASSIGN_ROLE,
        )
        .await
        .is_ok());
    assert!(authz
        .enforce_permission(
            &admin_subject,
            CoreAccessObject::user(UserId::new()),
            CoreAccessAction::USER_ASSIGN_ROLE,
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
    let (access, superuser_subject) = helpers::init_access(&pool, &authz).await?;

    let bank_manager_role = access
        .find_role_by_name(&superuser_subject, ROLE_NAME_BANK_MANAGER)
        .await?;

    let bank_manager_subject =
        create_user_with_role(&access, &superuser_subject, bank_manager_role.id).await?;

    // Bank Manager cannot create users
    assert!(matches!(
        authz
            .enforce_permission(
                &bank_manager_subject,
                CoreAccessObject::all_users(),
                CoreAccessAction::USER_CREATE,
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    // Bank Manager cannot assign roles
    assert!(matches!(
        authz
            .enforce_permission(
                &bank_manager_subject,
                CoreAccessObject::all_users(),
                CoreAccessAction::USER_ASSIGN_ROLE,
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    Ok(())
}
