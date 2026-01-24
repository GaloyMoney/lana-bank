mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use core_accounting::CoreAccounting;
use core_customer::Customers;
use core_deposit::*;
use document_storage::DocumentStorage;
use domain_config::InternalDomainConfigs;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use helpers::{BASE_ACCOUNTS_CSV, action, default_accounting_base_config, event, object};

const DEPOSIT_ACCOUNTS_CSV: &str = r#"
11,,,Omnibus Parent,,
21,,,Individual Deposit Accounts,,
22,,,Private Company Deposit Accounts,,
23,,,Bank Deposit Accounts,,
24,,,Financial Institution Deposit Accounts,,
25,,,Non Domiciled Individual Deposit Accounts,,
26,,,Government Entity Deposit Accounts,,
27,,,Frozen Deposit Accounts,,
"#;

#[tokio::test]
async fn chart_of_accounts_integration() -> anyhow::Result<()> {
    use rand::Rng;

    let pool = helpers::init_pool().await?;
    let (clock, _) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox =
        obix::Outbox::<event::DummyEvent>::init(&pool, obix::MailboxConfig::builder().build()?)
            .await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let domain_configs = InternalDomainConfigs::new(&pool);
    let governance = governance::Governance::new(&pool, &authz, &outbox, clock.clone());

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let mut jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .build()
            .unwrap(),
    )
    .await?;

    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let journal_id = helpers::init_journal(&cala).await?;
    let public_ids = public_id::PublicIds::new(&pool);

    let customers = Customers::new(
        &pool,
        &authz,
        &outbox,
        document_storage.clone(),
        public_ids.clone(),
        clock.clone(),
    );

    let exposed_domain_configs = helpers::init_domain_configs(&pool, &authz).await?;

    let deposit = CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &mut jobs,
        &cala,
        journal_id,
        &public_ids,
        &customers,
        &exposed_domain_configs,
    )
    .await?;

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
    let import = format!("{}{}", BASE_ACCOUNTS_CSV, DEPOSIT_ACCOUNTS_CSV);
    let base_config = default_accounting_base_config();
    let (chart, _) = accounting
        .chart_of_accounts()
        .import_from_csv_with_base_config(&DummySubject, &chart_ref, import, base_config.clone())
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
                chart_of_accounts_omnibus_parent_code: "11".parse().unwrap(),
                chart_of_accounts_individual_deposit_accounts_parent_code: "21".parse().unwrap(),
                chart_of_accounts_government_entity_deposit_accounts_parent_code: "26"
                    .parse()
                    .unwrap(),
                chart_of_account_private_company_deposit_accounts_parent_code: "22"
                    .parse()
                    .unwrap(),
                chart_of_account_bank_deposit_accounts_parent_code: "23".parse().unwrap(),
                chart_of_account_financial_institution_deposit_accounts_parent_code: "24"
                    .parse()
                    .unwrap(),
                chart_of_account_non_domiciled_individual_deposit_accounts_parent_code: "25"
                    .parse()
                    .unwrap(),
                chart_of_accounts_frozen_individual_deposit_accounts_parent_code: "27"
                    .parse()
                    .unwrap(),
                chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: "27"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_private_company_deposit_accounts_parent_code: "27"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_bank_deposit_accounts_parent_code: "27".parse().unwrap(),
                chart_of_account_frozen_financial_institution_deposit_accounts_parent_code: "27"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code: "27"
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
    let import = format!(
        "{}{}",
        BASE_ACCOUNTS_CSV,
        r#"
    11,,,Other Omnibus Parent,,
    21,,,Other Individual Deposit Accounts,,
    26,,,Other Government Entity Deposit Accounts,,
    22,,,Other Private Company Deposit Accounts,,
    23,,,Other Bank Deposit Accounts,,
    24,,,Other Financial Institution Deposit Accounts,,
    25,,,Other Non Domiciled Individual Deposit Accounts,,
    27,,,Other Frozen Deposit Accounts,,
    "#
    );
    let (chart, _) = accounting
        .chart_of_accounts()
        .import_from_csv_with_base_config(&DummySubject, &chart_ref, import, base_config)
        .await?;

    let res = deposit
        .set_chart_of_accounts_integration_config(
            &DummySubject,
            &chart,
            ChartOfAccountsIntegrationConfig {
                chart_of_accounts_id: chart_id,
                chart_of_accounts_omnibus_parent_code: "11".parse().unwrap(),
                chart_of_accounts_individual_deposit_accounts_parent_code: "21".parse().unwrap(),
                chart_of_accounts_government_entity_deposit_accounts_parent_code: "26"
                    .parse()
                    .unwrap(),
                chart_of_account_private_company_deposit_accounts_parent_code: "22"
                    .parse()
                    .unwrap(),
                chart_of_account_bank_deposit_accounts_parent_code: "23".parse().unwrap(),
                chart_of_account_financial_institution_deposit_accounts_parent_code: "24"
                    .parse()
                    .unwrap(),
                chart_of_account_non_domiciled_individual_deposit_accounts_parent_code: "25"
                    .parse()
                    .unwrap(),
                chart_of_accounts_frozen_individual_deposit_accounts_parent_code: "27"
                    .parse()
                    .unwrap(),
                chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: "27"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_private_company_deposit_accounts_parent_code: "27"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_bank_deposit_accounts_parent_code: "27".parse().unwrap(),
                chart_of_account_frozen_financial_institution_deposit_accounts_parent_code: "27"
                    .parse()
                    .unwrap(),
                chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code: "27"
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
