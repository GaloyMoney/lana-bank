mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use chart_of_accounts::new::CoreChartOfAccounts;
use deposit::*;
use helpers::{action, event, object};

#[tokio::test]
async fn chart_of_accounts_integration() -> anyhow::Result<()> {
    use rand::Rng;

    let pool = helpers::init_pool().await?;

    let outbox = outbox::Outbox::<event::DummyEvent>::init(&pool).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();

    let governance = governance::Governance::new(&pool, &authz, &outbox);
    let customers = core_customer::Customers::new(&pool, &authz, &outbox);

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
        &customers,
        &jobs,
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
        1,Deposit Parent
        2,Omnibus Parent
        "#
    .to_string();
    let chart_id = chart.id;
    let chart = charts
        .import_from_csv(&DummySubject, chart_id, import)
        .await?;

    let code = "1".parse::<chart_of_accounts::new::AccountCode>().unwrap();
    let account_set_id = cala
        .account_sets()
        .find(chart.account_spec(&code).unwrap().1)
        .await?
        .id;

    deposit
        .update_chart_of_accounts_integration_config(
            &DummySubject,
            chart,
            ChartOfAccountsIntegrationConfig::builder()
                .chart_of_accounts_id(chart_id)
                .chart_of_accounts_deposit_accounts_parent_code("1".parse().unwrap())
                .chart_of_accounts_omnibus_parent_code("2".parse().unwrap())
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
        1,Other Deposit Parent
        2,Other Omnibus Parent
        "#
    .to_string();
    let chart_id = chart.id;
    let chart = charts
        .import_from_csv(&DummySubject, chart_id, import)
        .await?;

    let code = "1".parse::<chart_of_accounts::new::AccountCode>().unwrap();
    let account_set_id = cala
        .account_sets()
        .find(chart.account_spec(&code).unwrap().1)
        .await?
        .id;

    deposit
        .update_chart_of_accounts_integration_config(
            &DummySubject,
            chart,
            ChartOfAccountsIntegrationConfig::builder()
                .chart_of_accounts_id(chart_id)
                .chart_of_accounts_deposit_accounts_parent_code("1".parse().unwrap())
                .chart_of_accounts_omnibus_parent_code("2".parse().unwrap())
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
