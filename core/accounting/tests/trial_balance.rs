mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use chrono::Utc;
use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use job::{JobSvcConfig, Jobs};

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
    let document_storage = DocumentStorage::new(&pool, &storage);
    let jobs = Jobs::init(JobSvcConfig::builder().pool(pool.clone()).build().unwrap()).await?;
    let domain_configs = domain_config::DomainConfigs::new(&pool);

    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        document_storage,
        &jobs,
        &domain_configs,
    );
    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    let chart_id = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?
        .id;
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
    let new_account_set_ids = accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, &chart_ref, import)
        .await?
        .1
        .unwrap();

    let trial_balance_name = format!("Trial Balance #{:05}", rand::rng().random_range(0..100000));
    accounting
        .trial_balances()
        .create_trial_balance_statement(trial_balance_name.to_string())
        .await?;

    let accounts = accounting
        .list_all_account_flattened(
            &DummySubject,
            &chart_ref,
            Utc::now().date_naive(),
            Some(Utc::now().date_naive()),
        )
        .await?;
    assert_eq!(accounts.len(), 0);

    accounting
        .trial_balances()
        .add_new_chart_accounts_to_trial_balance(&trial_balance_name, &new_account_set_ids)
        .await?;

    let chart = accounting.chart_of_accounts().find_by_id(chart_id).await?;
    let accounts = accounting
        .ledger_accounts()
        .list_all_account_flattened(
            &DummySubject,
            &chart,
            Utc::now().date_naive(),
            Some(Utc::now().date_naive()),
            false,
        )
        .await?;
    assert_eq!(accounts.len(), 6);

    Ok(())
}
