mod helpers;
use serial_test::file_serial;

use lana_app::{audit::*, authorization::init as init_authz};

#[tokio::test]
#[file_serial]
async fn roles_and_permission_sets_interaction() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let audit = Audit::new(&pool);
    let authz = init_authz(&pool, &audit).await?;
    let (users, superuser_subject) = helpers::init_users(&pool, &authz).await?;

    let roles = users.roles().list(&superuser_subject).await?;
    assert!(roles.iter().any(|r| r.name.name() == "superuser"));

    Ok(())
}
