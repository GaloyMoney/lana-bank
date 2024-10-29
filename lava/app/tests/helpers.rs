use lava_app::{authorization::Authorization, outbox::Outbox, primitives::Subject, user::Users};

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://user:password@{pg_host}:5433/pg");
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub async fn init_users(
    pool: &sqlx::PgPool,
    authz: &Authorization,
) -> anyhow::Result<(Users, Subject)> {
    let superuser_email = "superuser@test.io".to_string();
    let outbox = Outbox::init(&pool).await?;
    let users = Users::init(&pool, &authz, &outbox, Some(superuser_email.clone())).await?;
    let superuser = users
        .find_by_email(None, &superuser_email)
        .await?
        .expect("Superuser not found");
    Ok((users, Subject::from(superuser.id)))
}
