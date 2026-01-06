mod helpers;

use authz::dummy::{DummyPerms, DummySubject};
use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use domain_config::DomainConfigs;
use job::{JobSvcConfig, Jobs};

use cala_ledger::{
    CalaLedger, CalaLedgerConfig, Currency, DebitOrCredit, error::LedgerError,
    velocity::error::VelocityError,
};
use core_accounting::{
    AccountIdOrCode, Chart, CoreAccounting, ManualEntryInput,
    error::CoreAccountingError,
    manual_transaction::{
        error::ManualTransactionError, ledger::error::ManualTransactionLedgerError,
    },
};
use helpers::{action, object};
use rust_decimal_macros::dec;

#[tokio::test]
async fn manual_transaction_fails_for_unopened_fiscal_year() -> anyhow::Result<()> {
    let (accounting, chart) = prepare_test().await?;

    let to: AccountIdOrCode = "1".parse().unwrap();
    let from: AccountIdOrCode = "2".parse().unwrap();

    let entries = vec![
        ManualEntryInput::builder()
            .account_id_or_code(to.clone())
            .amount(dec!(100))
            .currency(Currency::USD)
            .direction(DebitOrCredit::Debit)
            .description("test 1 debit")
            .build()
            .unwrap(),
        ManualEntryInput::builder()
            .account_id_or_code(from.clone())
            .amount(dec!(100))
            .currency(Currency::USD)
            .direction(DebitOrCredit::Credit)
            .description("test 1 credit")
            .build()
            .unwrap(),
    ];
    let res = accounting
        .execute_manual_transaction(
            &DummySubject,
            &chart.reference,
            None,
            "Test transaction 1".to_string(),
            None,
            entries,
        )
        .await;

    assert!(matches!(
        res,
        Err(CoreAccountingError::ManualTransactionError(
            ManualTransactionError::ManualTransactionLedgerError(
                ManualTransactionLedgerError::CalaLedger(LedgerError::VelocityError(
                    VelocityError::Enforcement(_)
                ))
            )
        ))
    ));

    Ok(())
}

#[tokio::test]
async fn manual_transaction() -> anyhow::Result<()> {
    let (accounting, chart) = prepare_test().await?;

    let opened_as_of = "2021-01-01".parse::<chrono::NaiveDate>().unwrap();
    accounting
        .fiscal_year()
        .init_for_chart(&DummySubject, opened_as_of, chart.id)
        .await?;

    let to: AccountIdOrCode = "1".parse().unwrap();
    let from: AccountIdOrCode = "2".parse().unwrap();

    let entries = vec![
        ManualEntryInput::builder()
            .account_id_or_code(to.clone())
            .amount(dec!(100))
            .currency(Currency::USD)
            .direction(DebitOrCredit::Debit)
            .description("test 1 debit")
            .build()
            .unwrap(),
        ManualEntryInput::builder()
            .account_id_or_code(from.clone())
            .amount(dec!(100))
            .currency(Currency::USD)
            .direction(DebitOrCredit::Credit)
            .description("test 1 credit")
            .build()
            .unwrap(),
    ];
    accounting
        .execute_manual_transaction(
            &DummySubject,
            &chart.reference,
            None,
            "Test transaction 1".to_string(),
            None,
            entries,
        )
        .await?;

    let account = accounting
        .find_ledger_account_by_code(&DummySubject, &chart.reference, "2".to_string())
        .await?
        .unwrap();
    assert_eq!(
        account
            .usd_balance_range
            .expect("should have balance")
            .close
            .expect("balance missing")
            .settled(),
        dec!(100)
    );

    Ok(())
}

#[tokio::test]
async fn ledger_transactions_by_template_code() -> anyhow::Result<()> {
    let (accounting, chart) = prepare_test().await?;

    let opened_as_of = "2021-01-01".parse::<chrono::NaiveDate>().unwrap();
    accounting
        .fiscal_year()
        .init_for_chart(&DummySubject, opened_as_of, chart.id)
        .await?;

    let to: AccountIdOrCode = "1".parse().unwrap();
    let from: AccountIdOrCode = "2".parse().unwrap();

    let entries = vec![
        ManualEntryInput::builder()
            .account_id_or_code(to.clone())
            .amount(dec!(100))
            .currency(Currency::USD)
            .direction(DebitOrCredit::Debit)
            .description("test 1 debit")
            .build()
            .unwrap(),
        ManualEntryInput::builder()
            .account_id_or_code(from.clone())
            .amount(dec!(100))
            .currency(Currency::USD)
            .direction(DebitOrCredit::Credit)
            .description("test 1 credit")
            .build()
            .unwrap(),
    ];

    let manual_tx = accounting
        .execute_manual_transaction(
            &DummySubject,
            &chart.reference,
            None,
            "Test transaction 1".to_string(),
            None,
            entries,
        )
        .await?;

    let template_txs = accounting
        .ledger_transactions()
        .list_for_template_code(&DummySubject, "MANUAL_TRANSACTION_2", Default::default())
        .await?
        .entities;
    assert!(template_txs.iter().any(|tx| tx.id == manual_tx.id));

    Ok(())
}

async fn prepare_test() -> anyhow::Result<(
    CoreAccounting<DummyPerms<action::DummyAction, object::DummyObject>>,
    Chart,
)> {
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
    let chart = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?;
    let import = r#"
        1,,Assets
        2,,Liabilities
        "#;
    accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, &chart_ref, import)
        .await?;

    Ok((accounting, chart))
}
