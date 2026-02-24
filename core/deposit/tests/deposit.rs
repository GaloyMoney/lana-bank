mod helpers;

use rust_decimal_macros::dec;
use uuid::Uuid;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use core_customer::{CustomerType, Customers};
use core_deposit::*;
use document_storage::DocumentStorage;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use helpers::{action, event, object};
use obix::test_utils::expect_event;

async fn setup() -> anyhow::Result<(
    CoreDeposit<
        authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>,
        event::DummyEvent,
    >,
    Customers<
        authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>,
        event::DummyEvent,
    >,
    obix::Outbox<event::DummyEvent>,
    job::Jobs,
)> {
    let pool = helpers::init_pool().await?;
    let (clock, _time) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
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
        document_storage,
        public_ids.clone(),
        clock.clone(),
    );

    let exposed_domain_configs =
        helpers::init_read_only_exposed_domain_configs(&pool, &authz).await?;
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

    Ok((deposit, customers, outbox, jobs))
}

#[tokio::test]
async fn deposit() -> anyhow::Result<()> {
    let (deposit, customers, _outbox, _jobs) = setup().await?;

    let customer = customers
        .create_customer_bypassing_kyc(
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
    let (deposit, customers, _outbox, _jobs) = setup().await?;

    let customer = customers
        .create_customer_bypassing_kyc(
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

/// `DepositAccountCreated` is published when a new deposit account is created via
/// `CoreDeposit::create_account()`.
///
/// This event is consumed by `lana` notifications to send deposit-account-created emails.
///
/// The event contains a snapshot with the deposit account id and account holder id.
#[tokio::test]
async fn deposit_account_created_publishes_event() -> anyhow::Result<()> {
    let (deposit, customers, outbox, _jobs) = setup().await?;

    let customer = customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            format!("user{}@example.com", Uuid::new_v4()),
            format!("telegram{}", Uuid::new_v4()),
            CustomerType::Individual,
        )
        .await?;

    let (account, recorded) = expect_event(
        &outbox,
        || deposit.create_account(&DummySubject, customer.id),
        |result, e| match e {
            CoreDepositEvent::DepositAccountCreated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, account.id);
    assert_eq!(recorded.account_holder_id, account.account_holder_id);

    Ok(())
}

/// `DepositInitialized` is published when a deposit is recorded via `CoreDeposit::record_deposit()`.
///
/// This event is consumed by `lana` deposit sync (SumSub export) and `lana` customer sync (update last activity date).
///
/// The event contains a snapshot with the deposit id, deposit account id, and amount.
#[tokio::test]
async fn deposit_initialized_publishes_event() -> anyhow::Result<()> {
    let (deposit, customers, outbox, _jobs) = setup().await?;

    let customer = customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            format!("user{}@example.com", Uuid::new_v4()),
            format!("telegram{}", Uuid::new_v4()),
            CustomerType::Individual,
        )
        .await?;

    let account = deposit.create_account(&DummySubject, customer.id).await?;
    let amount = UsdCents::try_from_usd(dec!(1000000)).unwrap();

    let (deposit_record, recorded) = expect_event(
        &outbox,
        || deposit.record_deposit(&DummySubject, account.id, amount, None),
        |result, e| match e {
            CoreDepositEvent::DepositInitialized { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, deposit_record.id);
    assert_eq!(
        recorded.deposit_account_id,
        deposit_record.deposit_account_id
    );
    assert_eq!(recorded.amount, deposit_record.amount);

    Ok(())
}

/// `WithdrawalConfirmed` is published when a withdrawal is confirmed via
/// `CoreDeposit::confirm_withdrawal()`.
///
/// This event is consumed by `lana` deposit sync (SumSub export) and `lana` customer sync (update last activity date).
///
/// The event contains a snapshot with the withdrawal id, deposit account id, and amount.
///
/// This test requires the job poller because withdrawal approval is processed asynchronously
/// via the governance → outbox → jobs pipeline.
#[tokio::test]
#[serial_test::file_serial(job_poller)]
async fn withdrawal_confirmed_publishes_event() -> anyhow::Result<()> {
    let (deposit, customers, outbox, mut jobs) = setup().await?;
    jobs.start_poll().await?;

    let customer = customers
        .create_customer_bypassing_kyc(
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

    let withdrawal_amount = UsdCents::try_from_usd(dec!(500000)).unwrap();
    let withdrawal = deposit
        .initiate_withdrawal(&DummySubject, account.id, withdrawal_amount, None)
        .await?;

    // Withdrawal approval is concluded asynchronously via the governance → outbox → jobs pipeline.
    // This test wants to validate the WithdrawalConfirmed outbox event, so we must wait until the
    // approval job has written the ApprovalProcessConcluded event to the withdrawal before we can
    // successfully call confirm_withdrawal.
    let max_retries = 100;
    for attempt in 1..=max_retries {
        let Some(current) = deposit
            .find_withdrawal_by_id(&DummySubject, withdrawal.id)
            .await?
        else {
            anyhow::bail!("withdrawal not found");
        };
        match current.is_approved_or_denied() {
            Some(true) => break,
            Some(false) => anyhow::bail!("withdrawal approval was denied"),
            None => {}
        }
        if attempt == max_retries {
            anyhow::bail!("withdrawal approval not processed in time");
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let (confirmed, recorded) = expect_event(
        &outbox,
        || deposit.confirm_withdrawal(&DummySubject, withdrawal.id),
        |result, e| match e {
            CoreDepositEvent::WithdrawalConfirmed { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, confirmed.id);
    assert_eq!(recorded.deposit_account_id, confirmed.deposit_account_id);
    assert_eq!(recorded.amount, confirmed.amount);

    jobs.shutdown().await?;
    Ok(())
}

/// `DepositReverted` is published when a deposit is reverted via `CoreDeposit::revert_deposit()`.
///
/// This event is consumed by `lana` customer sync (update last activity date).
///
/// The event contains a snapshot with the deposit id, deposit account id, and amount.
#[tokio::test]
async fn deposit_reverted_publishes_event() -> anyhow::Result<()> {
    let (deposit, customers, outbox, _jobs) = setup().await?;

    let customer = customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            format!("user{}@example.com", Uuid::new_v4()),
            format!("telegram{}", Uuid::new_v4()),
            CustomerType::Individual,
        )
        .await?;

    let account = deposit.create_account(&DummySubject, customer.id).await?;
    let amount = UsdCents::try_from_usd(dec!(1000000)).unwrap();
    let deposit_record = deposit
        .record_deposit(&DummySubject, account.id, amount, None)
        .await?;

    let (reverted, recorded) = expect_event(
        &outbox,
        || deposit.revert_deposit(&DummySubject, deposit_record.id),
        |result, e| match e {
            CoreDepositEvent::DepositReverted { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, reverted.id);
    assert_eq!(recorded.deposit_account_id, reverted.deposit_account_id);
    assert_eq!(recorded.amount, reverted.amount);

    Ok(())
}
