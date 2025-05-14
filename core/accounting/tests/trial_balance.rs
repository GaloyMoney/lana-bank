mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use chrono::Utc;
use cloud_storage::{Storage, config::StorageConfig};
use job::{JobExecutorConfig, Jobs};

use core_accounting::*;
use helpers::{action, object};

#[tokio::test]
async fn add_chart_to_trial_balance() -> anyhow::Result<()> {
    use rand::Rng;

    let pool = helpers::init_pool().await?;
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let journal_id = helpers::init_journal(&cala).await?;

    let storage = Storage::new(&StorageConfig::default());
    let jobs = Jobs::new(&pool, JobExecutorConfig::default());

    let accounting = CoreAccounting::new(&pool, &authz, &cala, journal_id, &storage, &jobs);
    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    let chart = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?;
    let rand_ref = format!("{:05}", rand::rng().random_range(0..100000));
    let import = format!(
        r#"
        {rand_ref},,,Assets
        {rand_ref}1,,,Assets
        ,01,,Cash
        ,,0101,Central Office,
        ,02,,Payables
        ,,0101,Central Office,
        "#
    );
    let chart_id = chart.id;
    let chart = accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, chart_id, import)
        .await?
        .unwrap();

    let trial_balance_name = format!("Trial Balance #{:05}", rand::rng().random_range(0..100000));
    accounting
        .trial_balances()
        .create_trial_balance_statement(trial_balance_name.to_string())
        .await?;

    let trial_balance = accounting
        .trial_balances()
        .trial_balance(
            &DummySubject,
            trial_balance_name.to_string(),
            Utc::now().date_naive(),
            Utc::now().date_naive(),
        )
        .await?;

    let accounts = accounting
        .list_account_children(
            &DummySubject,
            &chart_ref,
            trial_balance.id,
            Default::default(),
            Utc::now().date_naive(),
            Some(Utc::now().date_naive()),
        )
        .await?;
    assert_eq!(accounts.entities.len(), 0);

    accounting
        .trial_balances()
        .add_chart_to_trial_balance(&trial_balance_name, &chart)
        .await?;

    let accounts = accounting
        .ledger_accounts()
        .list_account_children(
            &DummySubject,
            &chart,
            trial_balance.id,
            Default::default(),
            Utc::now().date_naive(),
            Some(Utc::now().date_naive()),
            false,
        )
        .await?;
    assert_eq!(accounts.entities.len(), 2);

    Ok(())
}
