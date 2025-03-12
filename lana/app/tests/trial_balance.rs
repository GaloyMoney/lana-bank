mod helpers;

use chrono::Utc;
use rand::Rng;

use lana_app::{authorization::Authorization, trial_balance::TrialBalances};

use cala_ledger::{CalaLedger, CalaLedgerConfig};

use chart_of_accounts::*;
use rbac_types::Subject;

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

pub async fn init_chart(
    pool: &sqlx::Pool<sqlx::Postgres>,
    authz: &Authorization,
    cala: &CalaLedger,
    journal_id: LedgerJournalId,
    subject: &Subject,
) -> anyhow::Result<Chart> {
    let chart_of_accounts = CoreChartOfAccounts::init(pool, authz, cala, journal_id).await?;

    let rand_ref = format!("{:05}", rand::thread_rng().gen_range(0..100000));
    let chart_id = chart_of_accounts
        .create_chart(subject, "Test Chart".to_string(), rand_ref.clone())
        .await?
        .id;

    let data = format!(
        r#"
        {rand_ref},,,Assets
        {rand_ref}1,,,Assets
        ,01,,Cash
        ,,0101,Central Office,
        ,02,,Payables
        ,,0101,Central Office,
        "#
    );

    Ok(chart_of_accounts
        .import_from_csv(subject, chart_id, data)
        .await?)
}

#[tokio::test]
async fn add_chart_to_trial_balance() -> anyhow::Result<()> {
    use lana_app::{audit::*, authorization::init as init_authz};

    let pool = init_pool().await?;

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let audit = Audit::new(&pool);
    let authz = init_authz(&pool, &audit).await?;
    let journal_id = init_journal(&cala).await?;
    let (_, superuser_subject) = helpers::init_users(&pool, &authz).await?;

    let trial_balance_name = format!(
        "Trial Balance #{:05}",
        rand::thread_rng().gen_range(0..100000)
    );
    let trial_balances = TrialBalances::init(&pool, &authz, &cala, journal_id).await?;
    trial_balances
        .create_trial_balance_statement(trial_balance_name.to_string())
        .await?;
    let trial_balance = trial_balances
        .trial_balance(
            &superuser_subject,
            trial_balance_name.to_string(),
            Utc::now(),
            None,
        )
        .await?;
    assert_eq!(trial_balance.accounts.len(), 0);

    let chart = init_chart(&pool, &authz, &cala, journal_id, &superuser_subject).await?;
    trial_balances
        .add_chart_to_trial_balance(trial_balance_name.to_string(), chart)
        .await?;
    let trial_balance = trial_balances
        .trial_balance(
            &superuser_subject,
            trial_balance_name.to_string(),
            Utc::now(),
            None,
        )
        .await?;
    assert_eq!(trial_balance.accounts.len(), 4);

    Ok(())
}
