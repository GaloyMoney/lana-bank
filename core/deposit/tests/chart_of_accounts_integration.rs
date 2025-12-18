mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use core_accounting::CoreAccounting;
use core_customer::Customers;
use core_deposit::*;
use document_storage::DocumentStorage;
use domain_config::DomainConfigs;
use helpers::{action, event, object};

#[tokio::test]
async fn chart_of_accounts_integration() -> anyhow::Result<()> {
    use rand::Rng;

    let pool = helpers::init_pool().await?;

    let outbox =
        obix::Outbox::<event::DummyEvent>::init(&pool, obix::MailboxConfig::default()).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let domain_configs = DomainConfigs::new(&pool);
    let governance = governance::Governance::new(&pool, &authz, &outbox);

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .build()
            .unwrap(),
    )
    .await?;

    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage);
    let journal_id = helpers::init_journal(&cala).await?;
    let public_ids = public_id::PublicIds::new(&pool);

    let customers = Customers::new(
        &pool,
        &authz,
        &outbox,
        document_storage.clone(),
        public_ids.clone(),
    );

    let deposit = CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &jobs,
        &cala,
        journal_id,
        &public_ids,
        &customers,
        DepositConfig {
            require_verified_customer_for_account: false,
        },
    )
    .await?;

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
    let import = r#"
        2,Omnibus Parent
        1,Individual Deposit Accounts
        7,Government Entity Deposit Accounts
        3,Private Company Deposit Accounts
        4,Bank Deposit Accounts
        5,Financial Institution Deposit Accounts
        6,Non Domiciled Individual Deposit Accounts
        8,Frozen Deposit Accounts
        "#
    .to_string();
    let (chart, _) = accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, &chart_ref, import)
        .await?;

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
            ChartOfAccountsIntegrationConfig {
                chart_of_accounts_id: chart_id,
                chart_of_accounts_omnibus_parent_code: "2".parse().unwrap(),
                chart_of_accounts_individual_deposit_accounts_parent_code: "1".parse().unwrap(),
                chart_of_accounts_government_entity_deposit_accounts_parent_code: "7"
                    .parse()
                    .unwrap(),
                chart_of_account_private_company_deposit_accounts_parent_code: "3".parse().unwrap(),
                chart_of_account_bank_deposit_accounts_parent_code: "4".parse().unwrap(),
                chart_of_account_financial_institution_deposit_accounts_parent_code: "5"
                    .parse()
                    .unwrap(),
                chart_of_account_non_domiciled_individual_deposit_accounts_parent_code: "6"
                    .parse()
                    .unwrap(),
                chart_of_accounts_frozen_individual_deposit_accounts_parent_code: "8"
                    .parse()
                    .unwrap(),
                chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: "8"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_private_company_deposit_accounts_parent_code: "8"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_bank_deposit_accounts_parent_code: "8".parse().unwrap(),
                chart_of_account_frozen_financial_institution_deposit_accounts_parent_code: "8"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code: "8"
                    .parse()
                    .unwrap(),
            },
        )
        .await?;

    let res = cala
        .account_sets()
        .list_members_by_created_at(account_set_id, Default::default())
        .await?;

    assert_eq!(res.entities.len(), 1);

    let chart_ref = format!("other-ref-{:08}", rand::rng().random_range(0..10000));
    let chart_id = accounting
        .chart_of_accounts()
        .create_chart(
            &DummySubject,
            "Other Test chart".to_string(),
            chart_ref.to_string(),
        )
        .await?
        .id;

    let import = r#"
        2,Other Omnibus Parent
        1,Other Individual Deposit Accounts
        7,Other Government Entity Deposit 
        3,Other Private Company Deposit Accounts
        4,Other Bank Deposit Accounts
        5,Other Financial Institution Deposit Accounts
        6,Other Non Domiciled Individual Deposit Accounts
        8,Other Frozen Deposit Accounts
        "#
    .to_string();
    let (chart, _) = accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, &chart_ref, import)
        .await?;

    let res = deposit
        .set_chart_of_accounts_integration_config(
            &DummySubject,
            &chart,
            ChartOfAccountsIntegrationConfig {
                chart_of_accounts_id: chart_id,
                chart_of_accounts_omnibus_parent_code: "2".parse().unwrap(),
                chart_of_accounts_individual_deposit_accounts_parent_code: "1".parse().unwrap(),
                chart_of_accounts_government_entity_deposit_accounts_parent_code: "7"
                    .parse()
                    .unwrap(),
                chart_of_account_private_company_deposit_accounts_parent_code: "3".parse().unwrap(),
                chart_of_account_bank_deposit_accounts_parent_code: "4".parse().unwrap(),
                chart_of_account_financial_institution_deposit_accounts_parent_code: "5"
                    .parse()
                    .unwrap(),
                chart_of_account_non_domiciled_individual_deposit_accounts_parent_code: "6"
                    .parse()
                    .unwrap(),
                chart_of_accounts_frozen_individual_deposit_accounts_parent_code: "8"
                    .parse()
                    .unwrap(),
                chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: "8"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_private_company_deposit_accounts_parent_code: "8"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_bank_deposit_accounts_parent_code: "8".parse().unwrap(),
                chart_of_account_frozen_financial_institution_deposit_accounts_parent_code: "8"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code: "8"
                    .parse()
                    .unwrap(),
            },
        )
        .await;

    assert!(matches!(
        res,
        Err(core_deposit::error::CoreDepositError::DepositConfigAlreadyExists)
    ));

    Ok(())
}
