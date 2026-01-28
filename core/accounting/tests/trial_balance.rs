mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use chrono::{TimeZone, Utc};
use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use job::{JobSvcConfig, Jobs};

use core_accounting::*;
use helpers::{BASE_ACCOUNTS_CSV, action, default_accounting_base_config, object};

#[tokio::test]
async fn atomic_import_adds_accounts_to_trial_balance() -> anyhow::Result<()> {
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
        &outbox,
    );

    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?;

    let (balance_sheet_name, pl_name, trial_balance_name) =
        helpers::create_test_statements(&accounting).await?;

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
    let chart = accounting
        .import_csv_with_base_config(
            &DummySubject,
            &chart_ref,
            import,
            base_config,
            &balance_sheet_name,
            &pl_name,
            &trial_balance_name,
        )
        .await?;

    let today = clock.today();
    let accounts = accounting
        .ledger_accounts()
        .list_all_account_flattened(&DummySubject, &chart, today, Some(today), false)
        .await?;
    // 9 base accounts + 5 additional accounts
    assert_eq!(accounts.len(), 9 + 5);

    Ok(())
}
