mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use chart_of_accounts::new::CoreChartOfAccounts;
use core_credit::*;
use helpers::{action, event, object};

#[tokio::test]
async fn chart_of_accounts_integration() -> anyhow::Result<()> {
    use rand::Rng;

    let pool = helpers::init_pool().await?;

    let outbox = outbox::Outbox::<event::DummyEvent>::init(&pool).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();

    let governance = governance::Governance::new(&pool, &authz, &outbox);
    let customers = core_customer::Customers::new(&pool, &authz, &outbox);
    let price = core_price::Price::new();

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let jobs = job::Jobs::new(&pool, job::JobExecutorConfig::default());

    let journal_id = helpers::init_journal(&cala).await?;

    let credit = CreditFacilities::init(
        &pool,
        Default::default(),
        &governance,
        &jobs,
        &authz,
        &customers,
        &price,
        &outbox,
        &cala,
        journal_id,
    )
    .await?;

    let charts = CoreChartOfAccounts::init(&pool, &authz, &cala, journal_id).await?;
    let chart_ref = format!("ref-{:08}", rand::thread_rng().gen_range(0..10000));
    let chart = charts
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref)
        .await?;
    let import = r#"
        1,Facility Omnibus Parent
        2,Collateral Omnibus Parent
        3,Facility Parent
        4,Collateral Parent
        5,Disbursed Receivable Parent
        6,Interest Receivable Parent
        7,Interest Income Parent
        8,Fee Income Parent
        "#
    .to_string();
    let chart_id = chart.id;
    let chart = charts
        .import_from_csv(&DummySubject, chart_id, import)
        .await?;

    let code = "1".parse::<chart_of_accounts::new::AccountCode>().unwrap();
    let account_set_id = cala
        .account_sets()
        .find(chart.account_set_id_from_code(&code).unwrap())
        .await?
        .id;

    credit
        .update_chart_of_accounts_integration_config(
            &DummySubject,
            chart,
            ChartOfAccountsIntegrationConfig::builder()
                .chart_of_accounts_id(chart_id)
                .chart_of_account_facility_omnibus_parent_code("1".parse().unwrap())
                .chart_of_account_collateral_omnibus_parent_code("2".parse().unwrap())
                .chart_of_account_facility_parent_code("3".parse().unwrap())
                .chart_of_account_collateral_parent_code("4".parse().unwrap())
                .chart_of_account_disbursed_receivable_parent_code("5".parse().unwrap())
                .chart_of_account_interest_receivable_parent_code("6".parse().unwrap())
                .chart_of_account_interest_income_parent_code("7".parse().unwrap())
                .chart_of_account_fee_income_parent_code("8".parse().unwrap())
                .build()
                .unwrap(),
        )
        .await?;

    let res = cala
        .account_sets()
        .list_members(account_set_id, Default::default())
        .await?;

    assert_eq!(res.entities.len(), 1);

    let first_account_set_id = account_set_id;

    let chart_ref = format!("other-ref-{:08}", rand::thread_rng().gen_range(0..10000));
    let chart = charts
        .create_chart(&DummySubject, "Other Test chart".to_string(), chart_ref)
        .await?;

    let import = r#"
        1,Other Facility Omnibus Parent
        2,Other Collateral Omnibus Parent
        3,Other Facility Parent
        4,Other Collateral Parent
        5,Other Disbursed Receivable Parent
        6,Other Interest Receivable Parent
        7,Other Interest Income Parent
        8,Other Fee Income Parent
        "#
    .to_string();
    let chart_id = chart.id;
    let chart = charts
        .import_from_csv(&DummySubject, chart_id, import)
        .await?;

    let code = "1".parse::<chart_of_accounts::new::AccountCode>().unwrap();
    let account_set_id = cala
        .account_sets()
        .find(chart.account_set_id_from_code(&code).unwrap())
        .await?
        .id;

    credit
        .update_chart_of_accounts_integration_config(
            &DummySubject,
            chart,
            ChartOfAccountsIntegrationConfig::builder()
                .chart_of_accounts_id(chart_id)
                .chart_of_account_facility_omnibus_parent_code("1".parse().unwrap())
                .chart_of_account_collateral_omnibus_parent_code("2".parse().unwrap())
                .chart_of_account_facility_parent_code("3".parse().unwrap())
                .chart_of_account_collateral_parent_code("4".parse().unwrap())
                .chart_of_account_disbursed_receivable_parent_code("5".parse().unwrap())
                .chart_of_account_interest_receivable_parent_code("6".parse().unwrap())
                .chart_of_account_interest_income_parent_code("7".parse().unwrap())
                .chart_of_account_fee_income_parent_code("8".parse().unwrap())
                .build()
                .unwrap(),
        )
        .await?;

    let res = cala
        .account_sets()
        .list_members(account_set_id, Default::default())
        .await?;

    assert_eq!(res.entities.len(), 1);

    let res = cala
        .account_sets()
        .list_members(first_account_set_id, Default::default())
        .await?;

    assert!(res.entities.is_empty());

    Ok(())
}
