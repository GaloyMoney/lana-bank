mod helpers;

use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use serde_json::json;
use uuid::Uuid;

use core_custody::{
    CoreCustodyEvent, CustodianId, CustodyPublisher, NewWallet, Wallet, WalletId, WalletNetwork,
    WalletRepo,
};
use core_money::Satoshis;
use helpers::event;

async fn setup() -> anyhow::Result<(
    WalletRepo<event::DummyEvent>,
    obix::Outbox<event::DummyEvent>,
    ClockHandle,
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

    let publisher = CustodyPublisher::new(&outbox);
    let wallets = WalletRepo::new(&pool, &publisher, clock.clone());

    Ok((wallets, outbox, clock))
}

async fn create_test_wallet(wallets: &WalletRepo<event::DummyEvent>) -> anyhow::Result<Wallet> {
    let new_wallet = NewWallet::builder()
        .id(WalletId::new())
        .custodian_id(CustodianId::new())
        .external_wallet_id(format!("external-wallet-{}", Uuid::new_v4()))
        .custodian_response(json!({}))
        .address(format!("tb1q{}", Uuid::new_v4().simple()))
        .network(WalletNetwork::Testnet3)
        .build()
        .expect("all fields for new wallet provided");

    let mut db = wallets.begin_op().await?;
    let wallet = wallets.create_in_op(&mut db, new_wallet).await?;
    db.commit().await?;

    Ok(wallet)
}

/// WalletBalanceUpdated event is published when a wallet balance changes.
///
/// The event contains a snapshot of the wallet, including:
/// - id: The wallet identifier
/// - address: The wallet address
/// - network: The wallet network
/// - balance: The updated balance (must be present)
/// - balance_updated_at: The time the balance was updated (must be present)
#[tokio::test]
async fn wallet_balance_updated_publishes_event() -> anyhow::Result<()> {
    let (wallets, outbox, clock) = setup().await?;

    let wallet = create_test_wallet(&wallets).await?;
    let wallet_id = wallet.id;

    let new_balance = Satoshis::from(50_000);
    let updated_at = clock.now();

    let (updated_wallet, recorded) = event::expect_event(
        &outbox,
        || async {
            let mut db = wallets.begin_op().await?;
            let mut wallet = wallets.find_by_id_in_op(&mut db, wallet_id).await?;
            if wallet.update_balance(new_balance, updated_at).did_execute() {
                wallets.update_in_op(&mut db, &mut wallet).await?;
            }
            db.commit().await?;
            Ok::<_, anyhow::Error>(wallet)
        },
        |result, e| match e {
            CoreCustodyEvent::WalletBalanceUpdated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, updated_wallet.id);
    assert_eq!(recorded.address, updated_wallet.address);
    assert_eq!(recorded.network, updated_wallet.network);
    assert_eq!(recorded.balance, updated_wallet.current_balance());
    assert_eq!(
        recorded.balance_updated_at,
        updated_wallet.last_balance_update()
    );

    Ok(())
}
