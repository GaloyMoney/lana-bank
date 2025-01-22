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
    let omnibus_code = journal_id.to_string();

    let chart_id = ChartId::new();
    let chart_of_accounts = CoreChartOfAccounts::init(&pool, &authz, &cala, journal_id).await?;
    chart_of_accounts
        .create_chart(
            chart_id,
            "Test Chart".to_string(),
            format!("{:02}", rand::thread_rng().gen_range(0..100)),
        )
        .await?;

    let control_account_path = chart_of_accounts
        .create_control_account(
            chart_id,
            ChartCategory::Liabilities,
            "Deposits".to_string(),
            "deposits".to_string(),
        )
        .await?;
    let control_sub_account_path = chart_of_accounts
        .create_control_sub_account(
            LedgerAccountSetId::new(),
            chart_id,
            control_account_path,
            "User Deposits".to_string(),
            "user-deposits".to_string(),
        )
        .await?;
    let factory = chart_of_accounts.transaction_account_factory(control_sub_account_path);

    let deposit = CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &jobs,
        factory,
        &cala,
        journal_id,
        omnibus_code,
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
