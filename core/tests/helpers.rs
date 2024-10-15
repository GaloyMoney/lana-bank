use lava_core::{
    audit::Audit,
    authorization::Authorization,
    data_export::Export,
    job::{JobExecutorConfig, Jobs},
    primitives::Subject,
    user::{User, UserConfig, Users},
};

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://user:password@{pg_host}:5433/pg");
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub async fn init_users(
    pool: &sqlx::PgPool,
    authz: &Authorization,
    audit: &Audit,
) -> anyhow::Result<(Users, User, Subject)> {
    let superuser_email = "superuser@test.io";
    let jobs = Jobs::new(pool, JobExecutorConfig::default());
    let export = Export::new("".to_string(), &jobs);
    let users = Users::init(
        pool,
        UserConfig {
            superuser_email: Some("superuser@test.io".to_string()),
        },
        authz,
        audit,
        &export,
    )
    .await?;
    let superuser = users
        .find_by_email(superuser_email)
        .await?
        .expect("Superuser not found");
    let superuser_subject = Subject::from(superuser.id);
    Ok((users, superuser, superuser_subject))
}
