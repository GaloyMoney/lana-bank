mod helpers;

use rand::Rng;
use std::collections::HashMap;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use core_accounting::{AccountCode, CalaAccountSetId, CoreAccounting};
use core_customer::Customers;
use core_deposit::*;
use document_storage::DocumentStorage;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use helpers::{
    BASE_ACCOUNTS_CSV, action, assert_attached_for_code, default_accounting_base_config, event,
    object, resolve_account_set_ids, resolve_omnibus_account_set_ids,
};

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

async fn assert_deposit_pairs(
    cala: &CalaLedger,
    chart: &core_accounting::Chart,
    account_set_ids: &HashMap<&'static str, CalaAccountSetId>,
    pairs: &[(&AccountCode, DepositSummaryAccountSetSpec)],
) -> anyhow::Result<()> {
    for (code, spec) in pairs {
        let id = *account_set_ids
            .get(spec.external_ref)
            .expect("missing deposit account set ref");
        assert_attached_for_code(cala, chart, code, id).await?;
    }
    Ok(())
}

async fn assert_omnibus_pairs(
    cala: &CalaLedger,
    chart: &core_accounting::Chart,
    account_set_ids: &HashMap<&'static str, CalaAccountSetId>,
    pairs: &[(&AccountCode, DepositOmnibusAccountSetSpec)],
) -> anyhow::Result<()> {
    for (code, spec) in pairs {
        let id = *account_set_ids
            .get(spec.account_set_ref)
            .expect("missing omnibus account set ref");
        assert_attached_for_code(cala, chart, code, id).await?;
    }
    Ok(())
}

#[tokio::test]
async fn chart_of_accounts_integration() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let (clock, _) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox =
        obix::Outbox::<event::DummyEvent>::init(&pool, obix::MailboxConfig::builder().build()?)
            .await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
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

    let exposed_domain_configs =
        helpers::init_read_only_exposed_domain_configs(&pool, &authz).await?;
    // Required to prevent the case there is an attempt to remove an account set member from
    // an account set that no longer exists.
    domain_config::DomainConfigTestUtils::clear_config_by_key(
        &pool,
        "deposit-chart-of-accounts-integration",
    )
    .await?;
    let internal_domain_configs = helpers::init_internal_domain_configs(&pool).await?;

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
        &internal_domain_configs,
    )
    .await?;

    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        document_storage,
        &mut jobs,
        &outbox,
    );
    let chart_ref = format!("ref-{:010}", rand::rng().random_range(0..10_000_000_000u64));
    let chart_id = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?
        .id;

    let (balance_sheet_name, pl_name, tb_name) =
        helpers::create_test_statements(&accounting).await?;

    let import = format!("{}{}", BASE_ACCOUNTS_CSV, DEPOSIT_ACCOUNTS_CSV);
    let base_config = default_accounting_base_config();
    let chart = accounting
        .import_csv_with_base_config(
            &DummySubject,
            &chart_ref,
            import,
            base_config.clone(),
            &balance_sheet_name,
            &pl_name,
            &tb_name,
        )
        .await?;

    let code = "1".parse::<core_accounting::AccountCode>().unwrap();
    let account_set_id = cala
        .account_sets()
        .find(chart.maybe_account_set_id_from_code(&code).unwrap())
        .await?
        .id;

    let chart_of_accounts_config = ChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart_id,
        chart_of_accounts_omnibus_parent_code: "11".parse().unwrap(),
        chart_of_accounts_individual_deposit_accounts_parent_code: "21".parse().unwrap(),
        chart_of_accounts_government_entity_deposit_accounts_parent_code: "26".parse().unwrap(),
        chart_of_account_private_company_deposit_accounts_parent_code: "22".parse().unwrap(),
        chart_of_account_bank_deposit_accounts_parent_code: "23".parse().unwrap(),
        chart_of_account_financial_institution_deposit_accounts_parent_code: "24".parse().unwrap(),
        chart_of_account_non_domiciled_company_deposit_accounts_parent_code: "25".parse().unwrap(),
        chart_of_accounts_frozen_individual_deposit_accounts_parent_code: "27".parse().unwrap(),
        chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: "27"
            .parse()
            .unwrap(),
        chart_of_account_frozen_private_company_deposit_accounts_parent_code: "27".parse().unwrap(),
        chart_of_account_frozen_bank_deposit_accounts_parent_code: "27".parse().unwrap(),
        chart_of_account_frozen_financial_institution_deposit_accounts_parent_code: "27"
            .parse()
            .unwrap(),
        chart_of_account_frozen_non_domiciled_company_deposit_accounts_parent_code: "27"
            .parse()
            .unwrap(),
    };

    deposit
        .chart_of_accounts_integrations()
        .set_config(&DummySubject, &chart, chart_of_accounts_config.clone())
        .await?;

    let catalog = DEPOSIT_ACCOUNT_SET_CATALOG;
    let deposit_account_set_ids =
        resolve_account_set_ids(&cala, journal_id, catalog.deposit_specs()).await?;
    let frozen_account_set_ids =
        resolve_account_set_ids(&cala, journal_id, catalog.frozen_specs()).await?;
    let omnibus_account_set_ids =
        resolve_omnibus_account_set_ids(&cala, journal_id, catalog.omnibus_specs()).await?;
    let deposit_catalog = catalog.deposit();
    let frozen = catalog.frozen();
    let omnibus = catalog.omnibus();

    let omnibus_pairs = [(
        &chart_of_accounts_config.chart_of_accounts_omnibus_parent_code,
        *omnibus,
    )];
    assert_omnibus_pairs(&cala, &chart, &omnibus_account_set_ids, &omnibus_pairs).await?;

    let deposit_pairs = [
        (
            &chart_of_accounts_config.chart_of_accounts_individual_deposit_accounts_parent_code,
            deposit_catalog.individual,
        ),
        (
            &chart_of_accounts_config
                .chart_of_accounts_government_entity_deposit_accounts_parent_code,
            deposit_catalog.government_entity,
        ),
        (
            &chart_of_accounts_config.chart_of_account_private_company_deposit_accounts_parent_code,
            deposit_catalog.private_company,
        ),
        (
            &chart_of_accounts_config.chart_of_account_bank_deposit_accounts_parent_code,
            deposit_catalog.bank,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_financial_institution_deposit_accounts_parent_code,
            deposit_catalog.financial_institution,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_non_domiciled_company_deposit_accounts_parent_code,
            deposit_catalog.non_domiciled_company,
        ),
    ];
    assert_deposit_pairs(&cala, &chart, &deposit_account_set_ids, &deposit_pairs).await?;

    let frozen_pairs = [
        (
            &chart_of_accounts_config
                .chart_of_accounts_frozen_individual_deposit_accounts_parent_code,
            frozen.individual,
        ),
        (
            &chart_of_accounts_config
                .chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code,
            frozen.government_entity,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_frozen_private_company_deposit_accounts_parent_code,
            frozen.private_company,
        ),
        (
            &chart_of_accounts_config.chart_of_account_frozen_bank_deposit_accounts_parent_code,
            frozen.bank,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_frozen_financial_institution_deposit_accounts_parent_code,
            frozen.financial_institution,
        ),
        (
            &chart_of_accounts_config
                .chart_of_account_frozen_non_domiciled_company_deposit_accounts_parent_code,
            frozen.non_domiciled_company,
        ),
    ];
    assert_deposit_pairs(&cala, &chart, &frozen_account_set_ids, &frozen_pairs).await?;

    let res = cala
        .account_sets()
        .list_members_by_created_at(account_set_id, Default::default())
        .await?;

    assert_eq!(res.entities.len(), 1);

    let chart_ref = format!(
        "other-ref-{:010}",
        rand::rng().random_range(0..10_000_000_000u64)
    );
    let chart_id = accounting
        .chart_of_accounts()
        .create_chart(
            &DummySubject,
            "Other Test chart".to_string(),
            chart_ref.to_string(),
        )
        .await?
        .id;

    let (balance_sheet_name2, pl_name2, tb_name2) =
        helpers::create_test_statements(&accounting).await?;

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
    let chart = accounting
        .import_csv_with_base_config(
            &DummySubject,
            &chart_ref,
            import,
            base_config,
            &balance_sheet_name2,
            &pl_name2,
            &tb_name2,
        )
        .await?;
    let chart_of_accounts_integration_config = ChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart_id,
        chart_of_accounts_omnibus_parent_code: "11".parse().unwrap(),
        chart_of_accounts_individual_deposit_accounts_parent_code: "21".parse().unwrap(),
        chart_of_accounts_government_entity_deposit_accounts_parent_code: "26".parse().unwrap(),
        chart_of_account_private_company_deposit_accounts_parent_code: "22".parse().unwrap(),
        chart_of_account_bank_deposit_accounts_parent_code: "23".parse().unwrap(),
        chart_of_account_financial_institution_deposit_accounts_parent_code: "24".parse().unwrap(),
        chart_of_account_non_domiciled_company_deposit_accounts_parent_code: "25".parse().unwrap(),
        chart_of_accounts_frozen_individual_deposit_accounts_parent_code: "27".parse().unwrap(),
        chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: "27"
            .parse()
            .unwrap(),
        chart_of_account_frozen_private_company_deposit_accounts_parent_code: "27".parse().unwrap(),
        chart_of_account_frozen_bank_deposit_accounts_parent_code: "27".parse().unwrap(),
        chart_of_account_frozen_financial_institution_deposit_accounts_parent_code: "27"
            .parse()
            .unwrap(),
        chart_of_account_frozen_non_domiciled_company_deposit_accounts_parent_code: "27"
            .parse()
            .unwrap(),
    };
    let res = deposit
        .chart_of_accounts_integrations()
        .set_config(
            &DummySubject,
            &chart,
            chart_of_accounts_integration_config.clone(),
        )
        .await
        .unwrap();

    assert_eq!(res, chart_of_accounts_integration_config);

    Ok(())
}
