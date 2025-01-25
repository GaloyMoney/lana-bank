mod helpers;

use rust_decimal_macros::dec;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use chart_of_accounts::{ChartCategory, CoreChartOfAccounts};
use deposit::*;
use helpers::{action, event, object};

#[tokio::test]
async fn deposit() -> anyhow::Result<()> {
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
    let control_sub_account = chart_of_accounts
        .create_control_sub_account(
            LedgerAccountSetId::new(),
            chart_id,
            control_account,
            "User Deposits".to_string(),
            "user-deposits".to_string(),
        )
        .await?;
    let factory = chart_of_accounts.transaction_account_factory(control_sub_account);

    let omnibus_control_account = chart_of_accounts
        .create_control_account(
            LedgerAccountSetId::new(),
            chart_id,
            ChartCategory::Assets,
            "Deposits Omnibus".to_string(),
            "deposits-omnibus".to_string(),
        )
        .await?;
    let omnibus_control_sub_account = chart_of_accounts
        .create_control_sub_account(
            LedgerAccountSetId::new(),
            chart_id,
            omnibus_control_account,
            "User Deposits Omnibus".to_string(),
            "user-deposits-omnibus".to_string(),
        )
        .await?;
    let omnibus_factory =
        chart_of_accounts.transaction_account_factory(omnibus_control_sub_account);

    let deposit = CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &jobs,
        factory,
        omnibus_factory,
        &cala,
        journal_id,
    )
    .await?;

    let account_holder_id = DepositAccountHolderId::new();
    let account = deposit
        .create_account(
            &DummySubject,
            account_holder_id,
            "Deposit for User #1",
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
