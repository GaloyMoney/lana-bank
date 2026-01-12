mod helpers;

use rust_decimal_macros::dec;
use uuid::Uuid;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use core_customer::{CustomerType, Customers};
use core_deposit::*;
use document_storage::DocumentStorage;

use helpers::{action, event, object};

#[tokio::test]
async fn overdraw_and_cancel_withdrawal() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;

    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .build()
            .expect("Couldn't build MailboxConfig"),
    )
    .await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let governance = governance::Governance::new(&pool, &authz, &outbox);

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
        &mut jobs,
        &cala,
        journal_id,
        &public_ids,
        &customers,
        DepositConfig {
            require_verified_customer_for_account: false,
        },
    )
    .await?;

    let customer = customers
        .create(
            &DummySubject,
            format!("user{}@example.com", Uuid::new_v4()),
            format!("telegram{}", Uuid::new_v4()),
            CustomerType::Individual,
        )
        .await?;

    let account = deposit.create_account(&DummySubject, customer.id).await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(1000000)).unwrap();

    deposit
        .record_deposit(&DummySubject, account.id, deposit_amount, None)
        .await?;

    // overdraw
    let withdrawal_amount = UsdCents::try_from_usd(dec!(5000000)).unwrap();
    let withdrawal = deposit
        .initiate_withdrawal(&DummySubject, account.id, withdrawal_amount, None)
        .await;
    assert!(matches!(
        withdrawal,
        Err(core_deposit::error::CoreDepositError::DepositLedgerError(_))
    ));

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
