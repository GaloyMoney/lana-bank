#![allow(dead_code)]

use rand::Rng;

use lana_app::{access::Access, authorization::Authorization, outbox::Outbox, primitives::Subject};

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://user:password@{pg_host}:5433/pg");
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
    let access = Access::init(
        pool,
        authz,
        &outbox,
        Some(superuser_email.clone()),
        &rbac_types::LanaAction::action_descriptions(),
    )
    .await?;
    let superuser = access
        .users()
        .find_by_email(None, &superuser_email)
        .await?
        .expect("Superuser not found");
    Ok((access, Subject::from(superuser.id)))
}
