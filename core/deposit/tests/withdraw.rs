mod helpers;

use rust_decimal_macros::dec;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use deposit::*;

use helpers::{action, event, object};

#[tokio::test]
async fn cancel_withdrawal() -> anyhow::Result<()> {
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

    let deposit = CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &jobs,
        &cala,
        journal_id,
        omnibus_code,
    )
    .await?;

    let account_holder_id = DepositAccountHolderId::new();
    let account = deposit
        .create_account(&DummySubject, account_holder_id)
        .await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(1000000)).unwrap();

    deposit
        .record_deposit(&DummySubject, account.id, deposit_amount, None)
        .await?;

    let withdrawal_amount = UsdCents::try_from_usd(dec!(500000)).unwrap();

    let withdrawal = deposit
        .initiate_withdrawal(&DummySubject, account.id, withdrawal_amount, None)
        .await?;

    let balance = deposit.account_balance(&DummySubject, account.id).await?;
    assert_eq!(balance.settled, deposit_amount - withdrawal_amount);
    assert_eq!(balance.pending, withdrawal_amount);

    deposit
        .cancel_withdrawal(&DummySubject, withdrawal.id)
        .await?;
    let balance = deposit.account_balance(&DummySubject, account.id).await?;
    assert_eq!(balance.settled, deposit_amount);

    Ok(())
}
