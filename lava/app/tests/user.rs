mod helpers;
use rand::distributions::{Alphanumeric, DistString};
use serial_test::file_serial;

use lava_app::{app::*, primitives::*, user::*};

fn generate_random_email() -> String {
    let random_string: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    format!("{}@example.com", random_string.to_lowercase())
}

#[tokio::test]
#[file_serial]
async fn bank_manager_lifecycle() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let superuser_email = generate_random_email();
    let user_config = UserConfig {
        superuser_email: Some(superuser_email.clone()),
    };
    let app_config = AppConfig {
        user: user_config,
        ..Default::default()
    };
    let app = LavaApp::run(pool, app_config).await?;

    let superuser = app.users().find_by_email(superuser_email).await?;
    let superuser_subject = Subject::from(superuser.unwrap().id);

    let user_email = generate_random_email();
    let user = app
        .users()
        .create_user(&superuser_subject, user_email.clone())
        .await?;
    assert_eq!(user.email, user_email);
    assert_eq!(user.current_roles().len(), 0);

    let bank_manager = app
        .users()
        .assign_role_to_user(&superuser_subject, user.id, Role::BankManager)
        .await
        .expect("Could not assign role to user");

    assert_eq!(bank_manager.id, user.id);
    let roles: Vec<_> = bank_manager.current_roles().into_iter().collect();
    assert_eq!(roles, vec![Role::BankManager]);

    let user = app
        .users()
        .revoke_role_from_user(&superuser_subject, bank_manager.id, Role::BankManager)
        .await?;

    assert_eq!(user.current_roles().len(), 0);

    Ok(())
}
