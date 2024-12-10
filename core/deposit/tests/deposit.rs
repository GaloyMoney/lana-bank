use rust_decimal_macros::dec;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use deposit::*;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://user:password@{pg_host}:5433/pg");
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub async fn init_journal(cala: &CalaLedger) -> anyhow::Result<cala_ledger::JournalId> {
    use cala_ledger::journal::*;

    let id = JournalId::new();
    let new = NewJournal::builder()
        .id(id)
        .name("Test journal")
        .build()
        .unwrap();
    let journal = cala.journals().create(new).await?;
    Ok(journal.id)
}

#[tokio::test]
async fn deposit() -> anyhow::Result<()> {
    let pool = init_pool().await?;

    let outbox = outbox::Outbox::<CoreDepositEvent>::init(&pool).await?;
    let authz = authz::dummy::DummyPerms::<CoreDepositAction, CoreDepositObject>::new();

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let journal_id = init_journal(&cala).await?;
    let omnibus_code = journal_id.to_string();

    let deposit =
        CoreDeposit::init(&pool, &authz, &outbox, &cala, journal_id, omnibus_code).await?;

    let account_holder_id = DepositAccountHolderId::new();
    let account = deposit
        .create_account(&DummySubject, account_holder_id)
        .await?;
    deposit
        .record_deposit(
            &DummySubject,
            account.id,
            UsdCents::try_from_usd(dec!(1000000)).unwrap(),
            None,
        )
        .await?;
    let balance = deposit.balance(&DummySubject, account.id).await?;

    // return zero when no deposit
    assert_eq!(balance, UsdCents::try_from_usd(dec!(1000000)).unwrap());
    Ok(())
}
