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

    let authz =
        authz::dummy::DummyPerms::<CoreChartOfAccountsAction, CoreChartOfAccountsObject>::new();

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let chart_of_accounts = CoreChartOfAccounts::init(&pool, &authz, &cala).await?;
    let chart_id = ChartId::new();
    chart_of_accounts
        .create_chart(&DummySubject, chart_id)
        .await?;

    let charts = chart_of_accounts.list_charts(&DummySubject).await?;
    assert!(charts.iter().any(|chart| chart.id == chart_id));

    let control_account_code = chart_of_accounts
        .create_control_account(
            chart_id,
            "10000000".parse()?,
            "Credit Facilities Receivable",
        )
        .await?;

    let control_sub_account_name = "Fixed-Term Credit Facilities Receivable";
    let control_sub_account_code = chart_of_accounts
        .create_control_sub_account(chart_id, control_account_code, control_sub_account_name)
        .await?;
    assert_eq!(
        control_sub_account_code.control_account(),
        Some(control_account_code)
    );

    Ok(())
}
