use authz::dummy::DummySubject;

use cala_ledger::{CalaLedger, CalaLedgerConfig};
use chart_of_accounts::*;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://user:password@{pg_host}:5433/pg");
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

#[tokio::test]
async fn chart_of_accounts() -> anyhow::Result<()> {
    let pool = init_pool().await?;

    let outbox = outbox::Outbox::<CoreChartOfAccountEvent>::init(&pool).await?;
    let authz =
        authz::dummy::DummyPerms::<CoreChartOfAccountAction, CoreChartOfAccountObject>::new();

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let chart_of_accounts = CoreChartOfAccount::init(&pool, &authz, &outbox, &cala).await?;
    let control_account_code = chart_of_accounts
        .create_control_account(
            &DummySubject,
            "10000000".parse()?,
            "Credit Facilities Receivable",
        )
        .await?;
    let control_sub_account_code = chart_of_accounts
        .create_control_sub_account(
            &DummySubject,
            control_account_code,
            "Fixed-Term Credit Facilities Receivable",
        )
        .await?;

    let transaction_account_name = "Fixed-Term Credit Facilities Receivable #1 for Customer 00-01";
    let transaction_account = chart_of_accounts
        .create_transaction_account(
            &DummySubject,
            control_sub_account_code,
            transaction_account_name,
            "",
        )
        .await?;

    let transaction_account = chart_of_accounts
        .find_account(&DummySubject, transaction_account.code)
        .await?
        .expect("Transaction account not found");
    assert_eq!(transaction_account.name, transaction_account_name);

    Ok(())
}
