use cala_ledger::{CalaLedger, CalaLedgerConfig};

use authz::dummy::DummySubject;
use chart_of_accounts::{
    CoreChartOfAccounts, CoreChartOfAccountsAction, CoreChartOfAccountsObject,
};

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
async fn import_from_csv() -> anyhow::Result<()> {
    use rand::Rng;

    let pool = init_pool().await?;
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let authz =
        authz::dummy::DummyPerms::<CoreChartOfAccountsAction, CoreChartOfAccountsObject>::new();
    let journal_id = init_journal(&cala).await?;

    let chart_of_accounts = CoreChartOfAccounts::init(&pool, &authz, &cala, journal_id).await?;

    let rand_ref = format!("{:05}", rand::thread_rng().gen_range(0..100000));
    let chart_id = chart_of_accounts
        .create_chart(&DummySubject, "Test Chart".to_string(), rand_ref.clone())
        .await?
        .id;

    let data = format!(
        r#"
        {rand_ref},,,Assets
        {rand_ref}1,,,Assets
        ,01,,Effective
        ,,0101,Central Office,
        "#
    );

    chart_of_accounts
        .import_from_csv(&DummySubject, chart_id, data)
        .await?;

    Ok(())
}
