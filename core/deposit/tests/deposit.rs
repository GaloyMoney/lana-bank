mod helpers;

use rust_decimal_macros::dec;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use chart_of_accounts::{
    new::CoreChartOfAccounts as NewChartOfAccounts, ChartCategory, CoreChartOfAccounts,
};
use deposit::*;
use helpers::{action, event, object};

#[tokio::test]
async fn deposit() -> anyhow::Result<()> {
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

    let chart_id = ChartId::new();
    let chart_of_accounts = CoreChartOfAccounts::init(&pool, &authz, &cala, journal_id).await?;
    chart_of_accounts
        .create_chart(
            chart_id,
            "Test Chart".to_string(),
            format!("{:02}", rand::thread_rng().gen_range(0..100)),
        )
        .await?;

    let control_account = chart_of_accounts
        .create_control_account(
            LedgerAccountSetId::new(),
            chart_id,
            ChartCategory::Liabilities,
            "Deposits".to_string(),
            "deposits".to_string(),
        )
        .await?;
    let deposits_name = "User Deposits";
    let deposits_reference = format!(
        "user-deposits:{:04}",
        rand::thread_rng().gen_range(0..10000)
    );
    let control_sub_account = chart_of_accounts
        .create_control_sub_account(
            chart_id,
            control_account,
            deposits_name.to_string(),
            deposits_reference,
        )
        .await?;

    let omnibus_control_account = chart_of_accounts
        .create_control_account(
            LedgerAccountSetId::new(),
            chart_id,
            ChartCategory::Assets,
            "Deposits Omnibus".to_string(),
            "deposits-omnibus".to_string(),
        )
        .await?;
    let omnibus_name = "User Deposits Omnibus";
    let omnibus_reference = &format!(
        "user-deposits-omnibus:{:04}",
        rand::thread_rng().gen_range(0..10000)
    );
    let omnibus_control_sub_account = chart_of_accounts
        .create_control_sub_account(
            chart_id,
            omnibus_control_account,
            omnibus_name.to_string(),
            omnibus_reference.to_string(),
        )
        .await?;
    let omnibus_factory =
        chart_of_accounts.transaction_account_factory(omnibus_control_sub_account);
    let omnibus_account_id = LedgerAccountId::new();
    let mut op = cala.begin_operation().await?;
    omnibus_factory
        .create_transaction_account_in_op(
            &mut op,
            omnibus_account_id,
            omnibus_reference,
            omnibus_name,
            omnibus_name,
        )
        .await?;
    op.commit().await?;

    let new_chart_of_accounts = NewChartOfAccounts::init(&pool, &authz, &cala, journal_id).await?;

    let deposit = CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &customers,
        &jobs,
        new_chart_of_accounts.leaf_account_factory(),
        DepositOmnibusAccountIds {
            deposits: omnibus_account_id,
        },
        &cala,
        journal_id,
    )
    .await?;

    let account_holder_id = DepositAccountHolderId::new();
    let account = deposit
        .create_account(
            &DummySubject,
            account_holder_id,
            &format!("user-deposit:{}", account_holder_id),
            &format!("Deposit for User {}", account_holder_id),
            "Deposit checking account for user.",
        )
        .await?;

    deposit
        .record_deposit(
            &DummySubject,
            account.id,
            UsdCents::try_from_usd(dec!(1000000)).unwrap(),
            None,
        )
        .await?;

    // NOTE: test when 0 balance
    let balance = deposit.account_balance(&DummySubject, account.id).await?;
    assert_eq!(
        balance.settled,
        UsdCents::try_from_usd(dec!(1000000)).unwrap()
    );

    Ok(())
}
