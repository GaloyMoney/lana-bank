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
    let superuser_email = format!(
        "superuser_{:05}@test.io",
        rand::rng().random_range(0..100000)
    );
    let outbox = Outbox::init(pool).await?;

    let config = AccessConfig {
        superuser_email: Some(superuser_email.clone()),
    };

    let access = Access::init(
        pool,
        config,
        rbac_types::LanaAction::action_descriptions(),
        seed::PREDEFINED_ROLES,
        authz,
        &outbox,
    )
    .await?;

    let superuser = access
        .users()
        .find_by_email(None, &superuser_email)
        .await?
        .expect("Superuser not found");

    Ok((access, Subject::from(superuser.id)))
}
