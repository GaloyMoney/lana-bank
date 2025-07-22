mod helpers;

use rust_decimal_macros::dec;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use core_deposit::*;
use helpers::{action, event, object};

#[tokio::test]
async fn deposit() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;

    let outbox = outbox::Outbox::<event::DummyEvent>::init(&pool).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let governance = governance::Governance::new(&pool, &authz, &outbox);

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let jobs = job::Jobs::new(&pool, job::JobsConfig::default());

    let journal_id = helpers::init_journal(&cala).await?;
    let public_ids = public_id::PublicIds::new(&pool);

    let deposit = CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &jobs,
        &cala,
        journal_id,
        &public_ids,
    )
    .await?;

    let account_holder_id = DepositAccountHolderId::new();
    let account = deposit
        .create_account(
            &DummySubject,
            account_holder_id,
            true,
            DepositAccountType::Individual,
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

#[tokio::test]
async fn revert_deposit() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;

    let outbox = outbox::Outbox::<event::DummyEvent>::init(&pool).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let governance = governance::Governance::new(&pool, &authz, &outbox);

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let jobs = job::Jobs::new(&pool, job::JobsConfig::default());

    let journal_id = helpers::init_journal(&cala).await?;
    let public_ids = public_id::PublicIds::new(&pool);

    let deposit = CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &jobs,
        &cala,
        journal_id,
        &public_ids,
    )
    .await?;

    let account_holder_id = DepositAccountHolderId::new();
    let account = deposit
        .create_account(
            &DummySubject,
            account_holder_id,
            true,
            DepositAccountType::Individual,
        )
        .await?;

    let res = deposit
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

    // revert deposit
    deposit.revert_deposit(&DummySubject, res.id).await?;
    let balance = deposit.account_balance(&DummySubject, account.id).await?;

    assert_eq!(balance.settled, UsdCents::ZERO);

    Ok(())
}
