mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{config::StorageConfig, Storage};
use core_accounting::CoreAccounting;
use deposit::*;
use helpers::{action, event, object};

#[tokio::test]
async fn chart_of_accounts_integration() -> anyhow::Result<()> {
    use rand::Rng;

    let pool = helpers::init_pool().await?;

    let outbox = outbox::Outbox::<event::DummyEvent>::init(&pool).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let governance = governance::Governance::new(&pool, &authz, &outbox);

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let jobs = job::Jobs::new(&pool, job::JobExecutorConfig::default());

    let journal_id = helpers::init_journal(&cala).await?;

    let deposit = CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &jobs,
        &cala,
        journal_id,
    )
    .await?;

    let storage = Storage::new(&StorageConfig::default());
    let accounting = CoreAccounting::new(&pool, &authz, &cala, journal_id, &storage, &jobs);
    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    let chart = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?;
    let import = r#"
        2,Omnibus Parent
        1,Individual Deposit Accounts
        7,Government Entity Deposit Accounts
        3,Private Company Deposit Accounts
        4,Bank Deposit Accounts
        5,Financial Institution Deposit Accounts
        6,Non Domiciled Individual Deposit Accounts
        "#
    .to_string();
    let chart_id = chart.id;
    accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, chart_id, import)
        .await?
        .unwrap();
    let chart = accounting
        .chart_of_accounts()
        .find_by_reference(&chart_ref)
        .await
        .unwrap()
        .unwrap();

    let code = "1".parse::<core_accounting::AccountCode>().unwrap();
    let account_set_id = cala
        .account_sets()
        .find(chart.account_set_id_from_code(&code).unwrap())
        .await?
        .id;

    deposit
        .set_chart_of_accounts_integration_config(
            &DummySubject,
            &chart,
            ChartOfAccountsIntegrationConfig::builder()
                .chart_of_accounts_id(chart_id)
                .chart_of_accounts_omnibus_parent_code("2".parse().unwrap())
                .chart_of_accounts_individual_deposit_accounts_parent_code("1".parse().unwrap())
                .chart_of_accounts_government_entity_deposit_accounts_parent_code(
                    "7".parse().unwrap(),
                )
                .chart_of_account_private_company_deposit_accounts_parent_code("3".parse().unwrap())
                .chart_of_account_bank_deposit_accounts_parent_code("4".parse().unwrap())
                .chart_of_account_financial_institution_deposit_accounts_parent_code(
                    "5".parse().unwrap(),
                )
                .chart_of_account_non_domiciled_individual_deposit_accounts_parent_code(
                    "6".parse().unwrap(),
                )
                .build()
                .unwrap(),
        )
        .await?;

    let res = cala
        .account_sets()
        .list_members_by_created_at(account_set_id, Default::default())
        .await?;

    assert_eq!(res.entities.len(), 1);

    let chart_ref = format!("other-ref-{:08}", rand::rng().random_range(0..10000));
    let chart = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Other Test chart".to_string(), chart_ref)
        .await?;

    let import = r#"
        2,Other Omnibus Parent
        1,Other Individual Deposit Accounts
        7,Other Government Entity Deposit 
        3,Other Private Company Deposit Accounts
        4,Other Bank Deposit Accounts
        5,Other Financial Institution Deposit Accounts
        6,Other Non Domiciled Individual Deposit Accounts
        "#
    .to_string();
    let chart_id = chart.id;
    accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, chart_id, import)
        .await?
        .unwrap();

    let res = deposit
        .set_chart_of_accounts_integration_config(
            &DummySubject,
            &chart,
            ChartOfAccountsIntegrationConfig::builder()
                .chart_of_accounts_id(chart_id)
                .chart_of_accounts_omnibus_parent_code("2".parse().unwrap())
                .chart_of_accounts_individual_deposit_accounts_parent_code("1".parse().unwrap())
                .chart_of_accounts_government_entity_deposit_accounts_parent_code(
                    "7".parse().unwrap(),
                )
                .chart_of_account_private_company_deposit_accounts_parent_code("3".parse().unwrap())
                .chart_of_account_bank_deposit_accounts_parent_code("4".parse().unwrap())
                .chart_of_account_financial_institution_deposit_accounts_parent_code(
                    "5".parse().unwrap(),
                )
                .chart_of_account_non_domiciled_individual_deposit_accounts_parent_code(
                    "6".parse().unwrap(),
                )
                .build()
                .unwrap(),
        )
        .await;

    assert!(matches!(
        res,
        Err(deposit::error::CoreDepositError::DepositConfigAlreadyExists)
    ));

    Ok(())
}
