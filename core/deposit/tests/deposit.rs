use authz::dummy::DummySubject;
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
    let deposit = CoreDeposit::init(&pool, &authz, &outbox).await?;
    let account_holder_id = AccountHolderId::new();
    let account = deposit
        .create_account(&DummySubject, account_holder_id)
        .await?;
    deposit
        .record_deposit(&DummySubject, account.id, None)
        .await?;
    let _ = deposit.balance(&DummySubject, account.id).await?;
    Ok(())
}
