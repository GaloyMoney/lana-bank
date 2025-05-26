mod helpers;
use rand::distr::{Alphanumeric, SampleString};
use serial_test::file_serial;

use lana_app::{audit::*, authorization::init as init_authz, primitives::*};

fn generate_random_email() -> String {
    let random_string: String = Alphanumeric.sample_string(&mut rand::rng(), 32);
    format!("{}@example.com", random_string.to_lowercase())
}

#[tokio::test]
#[file_serial]
async fn bank_manager_lifecycle() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = init_authz(&pool, &audit).await?;
    let (access, superuser_subject) = helpers::init_access(&pool, &authz).await?;

    let user_email = generate_random_email();
    let user = access
        .users()
        .create_user(&superuser_subject, user_email.clone())
        .await?;
    assert_eq!(user.email, user_email);
    assert_eq!(user.current_role(), None);

    let bank_manager_role = access
        .find_role_by_name(&superuser_subject, LanaRole::BANK_MANAGER)
        .await?;

    let bank_manager = access
        .assign_role_to_user(&superuser_subject, user.id, bank_manager_role.id)
        .await
        .expect("Could not assign role to user");

    assert_eq!(bank_manager.id, user.id);
    assert_eq!(bank_manager.current_role(), Some(bank_manager_role.id));

    let user = access
        .users()
        .revoke_role_from_user(&superuser_subject, bank_manager.id, &bank_manager_role)
        .await?;

    assert_eq!(user.current_role(), None);

    Ok(())
}
