mod helpers;
use rand::distr::{Alphanumeric, SampleString};
use rbac_types::{ROLE_NAME_ACCOUNTANT, ROLE_NAME_ADMIN, ROLE_NAME_BANK_MANAGER};
use serial_test::file_serial;

use lana_app::{audit::*, authorization::Authorization};

fn generate_random_email() -> String {
    let random_string: String = Alphanumeric.sample_string(&mut rand::rng(), 32);
    format!("{}@example.com", random_string.to_lowercase())
}

#[tokio::test]
#[file_serial]
async fn bank_manager_lifecycle() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = Authorization::init(&pool, &audit).await?;
    let (access, superuser_subject) = helpers::init_access(&pool, &authz).await?;

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
