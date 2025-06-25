#![allow(dead_code)]

use rand::Rng;

use lana_app::{
    access::{Access, config::AccessConfig},
    authorization::{Authorization, seed},
    outbox::Outbox,
    primitives::Subject,
};

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON").unwrap();
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub async fn init_access(
    pool: &sqlx::PgPool,
    authz: &Authorization,
) -> anyhow::Result<(Access, Subject)> {
    let superuser_email = "superuser@test.io".to_string();
    let outbox = Outbox::init(pool).await?;

    let config = AccessConfig {
        superuser_email: Some(superuser_email.clone()),
        action_descriptions: rbac_types::LanaAction::action_descriptions(),
        predefined_roles: seed::PREDEFINED_ROLES,
    };

    let access = Access::init(pool, config, authz, &outbox).await?;

    let superuser = access
        .users()
        .find_by_email(None, &superuser_email)
        .await?
        .expect("Superuser not found");

    Ok((access, Subject::from(superuser.id)))
}
