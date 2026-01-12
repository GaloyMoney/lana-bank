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
async fn deposit() -> anyhow::Result<()> {
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
