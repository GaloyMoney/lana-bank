mod helpers;

use serial_test::file_serial;

use authz::PermissionCheck;

use lana_app::{
    access::Access,
    audit::*,
    authorization::{error::AuthorizationError, *},
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
        .create_user(superuser_subject, random_email(), role_id)
        .await?;
    Ok(Subject::from(user.id))
}

#[tokio::test]
#[file_serial]
async fn superuser_permissions() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let (_, superuser_subject) = helpers::init_access(&pool, &authz).await?;

    // Superuser can create users
    assert!(
        authz
            .enforce_permission(
                &superuser_subject,
                CoreAccessObject::all_users(),
                CoreAccessAction::USER_CREATE,
            )
            .await
            .is_ok()
    );

    // Superuser can assign Admin role
    assert!(
        authz
            .enforce_permission(
                &superuser_subject,
                CoreAccessObject::all_users(),
                CoreAccessAction::USER_UPDATE_ROLE,
            )
            .await
            .is_ok()
    );

    // Superuser can assign Bank Manager role
    assert!(
        authz
            .enforce_permission(
                &superuser_subject,
                CoreAccessObject::user(UserId::new()),
                CoreAccessAction::USER_UPDATE_ROLE,
            )
            .await
            .is_ok()
    );

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn admin_permissions() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let (access, superuser_subject) = helpers::init_access(&pool, &authz).await?;

    let admin_role = access
        .find_role_by_name(&superuser_subject, ROLE_NAME_ADMIN)
        .await?;

    let admin_subject = create_user_with_role(&access, &superuser_subject, admin_role.id).await?;

    // Admin can create users
    assert!(
        authz
            .enforce_permission(
                &admin_subject,
                CoreAccessObject::all_users(),
                CoreAccessAction::USER_CREATE,
            )
            .await
            .is_ok()
    );

    // Admin can assign roles
    assert!(
        authz
            .enforce_permission(
                &admin_subject,
                CoreAccessObject::all_users(),
                CoreAccessAction::USER_UPDATE_ROLE,
            )
            .await
            .is_ok()
    );
    assert!(
        authz
            .enforce_permission(
                &admin_subject,
                CoreAccessObject::user(UserId::new()),
                CoreAccessAction::USER_UPDATE_ROLE,
            )
            .await
            .is_ok()
    );

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn bank_manager_permissions() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
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
                CoreAccessAction::USER_UPDATE_ROLE,
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn custom_role_permissions() -> anyhow::Result<()> {
    const ROLE_NAME_CUSTOM_ROLE: &str = "custom-role";
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let (access, superuser_subject) = helpers::init_access(&pool, &authz).await?;

    // Creating a custom role with no initial permission sets
    let custom_role = access
        .create_role(
            &superuser_subject,
            ROLE_NAME_CUSTOM_ROLE.to_string(),
            Vec::<PermissionSetId>::new(),
        )
        .await?;

    let custom_role_subject =
        create_user_with_role(&access, &superuser_subject, custom_role.id).await?;

    // Assigning all permission sets of bank-manager role to the custom role
    let bank_manager_role = access
        .find_role_by_name(&superuser_subject, ROLE_NAME_BANK_MANAGER)
        .await?;

    access
        .add_permission_sets_to_role(
            &superuser_subject,
            custom_role.id,
            bank_manager_role.permission_sets().iter().copied(),
        )
        .await?;

    // The subject is now authorized to take actions which a bank manager can (like viewing credit facilities)
    assert!(
        authz
            .enforce_permission(
                &custom_role_subject,
                core_credit::CoreCreditObject::all_credit_facilities(),
                core_credit::CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await
            .is_ok()
    );

    // Revoking all permission sets assigned to the custom role (which were inherited from the bank-manager role)
    access
        .remove_permission_sets_from_role(
            &superuser_subject,
            custom_role.id,
            bank_manager_role.permission_sets().iter().copied(),
        )
        .await?;

    // The subject lost all permissions and authorization to take any actions (empty permission sets)
    assert!(matches!(
        authz
            .enforce_permission(
                &custom_role_subject,
                core_credit::CoreCreditObject::all_credit_facilities(),
                core_credit::CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await,
        Err(AuthorizationError::NotAuthorized)
    ));

    Ok(())
}
