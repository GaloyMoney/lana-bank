mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use core_accounting::CoreAccounting;
use document_storage::DocumentStorage;
use domain_config::DomainConfigs;
use helpers::{action, object};
use job::{JobSvcConfig, Jobs};

#[tokio::test]
async fn import_from_csv_creates_accounts() -> anyhow::Result<()> {
    use rand::Rng;

    let pool = helpers::init_pool().await?;
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let domain_configs = DomainConfigs::new(&pool);
    let journal_id = helpers::init_journal(&cala).await?;

    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage);
    let mut jobs = Jobs::init(JobSvcConfig::builder().pool(pool.clone()).build().unwrap()).await?;

    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        document_storage,
        &mut jobs,
        &domain_configs,
    );

    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?;

    let import = r#"
        1,,Assets
        2,,Liabilities
        "#;
    let (chart, control_accounts) = accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, &chart_ref, import)
        .await?;

    assert!(chart.account_set_id_from_code(&"1".parse()?).is_ok());
    assert!(chart.account_set_id_from_code(&"2".parse()?).is_ok());
    assert_eq!(
        control_accounts
            .expect("should have control accounts")
            .len(),
        2
    );

    Ok(())
}

#[tokio::test]
async fn import_from_csv_with_base_config_creates_accounts() -> anyhow::Result<()> {
    use core_accounting::AccountingBaseConfig;
    use rand::Rng;

    let pool = helpers::init_pool().await?;
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let domain_configs = DomainConfigs::new(&pool);
    let journal_id = helpers::init_journal(&cala).await?;

    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage);
    let mut jobs = Jobs::init(JobSvcConfig::builder().pool(pool.clone()).build().unwrap()).await?;

    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        document_storage,
        &mut jobs,
        &domain_configs,
    );

    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?;

    let import = r#"
        1,,Assets
        2,,Liabilities
        3,,Equity
        4,,Revenue
        5,,Cost of Revenue
        6,,Expenses
        "#;
    let (chart, _control_accounts) = accounting
        .chart_of_accounts()
        .import_from_csv_with_base_config(
            &DummySubject,
            &chart_ref,
            import,
            AccountingBaseConfig {
                assets_code: "1".parse()?,
                liabilities_code: "2".parse()?,
                equity_code: "3".parse()?,
                revenue_code: "4".parse()?,
                cost_of_revenue_code: "5".parse()?,
                expenses_code: "6".parse()?,
            },
        )
        .await?;

    assert!(chart.base_config.is_some());

    Ok(())
}
