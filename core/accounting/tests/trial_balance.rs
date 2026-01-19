mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use chrono::{TimeZone, Utc};
use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use domain_config::InternalDomainConfigs;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use job::{JobSvcConfig, Jobs};

use core_accounting::*;
use helpers::{BASE_ACCOUNTS_CSV, action, default_accounting_base_config, object};

#[tokio::test]
async fn add_chart_to_trial_balance() -> anyhow::Result<()> {
    use rand::Rng;

    let pool = helpers::init_pool().await?;
    let start_time = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let (clock, _ctrl) = ClockHandle::artificial(ArtificialClockConfig::manual_at(start_time));
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let domain_configs = InternalDomainConfigs::new(&pool);
    let journal_id = helpers::init_journal(&cala).await?;
    let outbox = helpers::init_outbox(&pool).await?;

    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let mut jobs = Jobs::init(JobSvcConfig::builder().pool(pool.clone()).build().unwrap()).await?;

    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        document_storage,
        &mut jobs,
        &domain_configs,
        &outbox,
    );
    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    let chart_id = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?
        .id;
    let rand_ref = format!("{:05}", rand::rng().random_range(0..100000));
    let import = format!(
        r#"{base}
    1{rand_ref},,,Current Assets,,
    ,01,,Cash,,
    ,,0101,Central Office,,
    ,02,,Payables,,
    ,,0101,Central Office,,
    "#,
        base = BASE_ACCOUNTS_CSV,
        rand_ref = rand_ref,
    );
    let base_config = default_accounting_base_config();
    let new_account_set_ids = accounting
        .chart_of_accounts()
        .import_from_csv_with_base_config(&DummySubject, &chart_ref, import, base_config)
        .await?
        .1
        .unwrap();

    let trial_balance_name = format!("Trial Balance #{:05}", rand::rng().random_range(0..100000));
    accounting
        .trial_balances()
        .create_trial_balance_statement(trial_balance_name.to_string())
        .await?;

    let today = clock.today();
    let accounts = accounting
        .list_all_account_flattened(&DummySubject, &chart_ref, today, Some(today))
        .await?;
    assert_eq!(accounts.len(), 0);

    accounting
        .trial_balances()
        .add_new_chart_accounts_to_trial_balance(&trial_balance_name, &new_account_set_ids)
        .await?;

    let chart = accounting.chart_of_accounts().find_by_id(chart_id).await?;
    let accounts = accounting
        .ledger_accounts()
        .list_all_account_flattened(&DummySubject, &chart, today, Some(today), false)
        .await?;
    assert_eq!(accounts.len(), 9 + 5);

    Ok(())
}
