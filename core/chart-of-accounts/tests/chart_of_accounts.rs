use authz::dummy::DummySubject;

use cala_ledger::{CalaLedger, CalaLedgerConfig};
use chart_of_accounts::*;

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
async fn create_and_populate() -> anyhow::Result<()> {
    use rand::Rng;

    let pool = init_pool().await?;

    let authz =
        authz::dummy::DummyPerms::<CoreChartOfAccountsAction, CoreChartOfAccountsObject>::new();

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let journal_id = init_journal(&cala).await?;

    let chart_of_accounts = CoreChartOfAccounts::init(&pool, &authz, &cala, journal_id).await?;
    let chart_id = ChartId::new();
    chart_of_accounts
        .create_chart(
            chart_id,
            "Test Chart".to_string(),
            format!("{:05}", rand::thread_rng().gen_range(0..100000)),
        )
        .await?;

    let charts = chart_of_accounts.list_charts(&DummySubject).await?;
    assert!(charts.iter().any(|chart| chart.id == chart_id));

    let control_account = chart_of_accounts
        .create_control_account(
            LedgerAccountSetId::new(),
            chart_id,
            ChartCategory::Assets,
            "Credit Facilities Receivable".to_string(),
            "credit-facilities-receivable".to_string(),
        )
        .await?;

    let control_sub_account_name = "Fixed-Term Credit Facilities Receivable";
    let control_sub_account = chart_of_accounts
        .create_control_sub_account(
            LedgerAccountSetId::new(),
            chart_id,
            control_account.clone(),
            control_sub_account_name.to_string(),
            "fixed-term-credit-facilities-receivable".to_string(),
        )
        .await?;
    assert_eq!(
        control_sub_account.path.control_account(),
        control_account.path
    );

    Ok(())
}

#[tokio::test]
async fn create_with_duplicate_reference() -> anyhow::Result<()> {
    use rand::Rng;

    use crate::LedgerJournalId;

    let pool = init_pool().await?;

    let authz =
        authz::dummy::DummyPerms::<CoreChartOfAccountsAction, CoreChartOfAccountsObject>::new();

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let chart_of_accounts =
        CoreChartOfAccounts::init(&pool, &authz, &cala, LedgerJournalId::new()).await?;

    let reference = format!("{:02}", rand::thread_rng().gen_range(0..100));

    let chart_id = ChartId::new();
    chart_of_accounts
        .create_chart(chart_id, "Test Chart".to_string(), reference.clone())
        .await?;
    let res = chart_of_accounts
        .create_chart(chart_id, "Test Chart".to_string(), reference.clone())
        .await;
    assert!(res.is_err());

    let chart = chart_of_accounts.find_by_reference(reference).await?;
    assert!(chart.is_some());

    Ok(())
}
