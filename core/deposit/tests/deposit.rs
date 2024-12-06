use deposit::*;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://user:password@{pg_host}:5433/pg");
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

#[tokio::test]
async fn deposit() -> anyhow::Result<()> {
    let pool = init_pool().await?;
    let outbox = outbox::Outbox::<CoreDepositEvent>::init(&pool).await?;
    let authz = authz::dummy::DummyPerms::<CoreDepositAction, CoreDepositObject>::new();
    let _deposit = CoreDeposit::init(&pool, &authz, &outbox).await?;
    Ok(())
}
