mod helpers;

use rand::RngExt;
use rust_decimal_macros::dec;

use core_price::{Price, PriceOfOneBTC};
use helpers::{DummyEvent, init_pool, publish_dummy_price_event, wait_for_price_to_be_updated};
use money::{Satoshis, UsdCents};
use obix::out::Outbox;

#[tokio::test]
async fn get_price_from_client() {
    let client = bfx_client::BfxClient::new();
    let tick = client.btc_usd_tick().await.expect("should fetch tick");
    let usd_cents = UsdCents::try_from_usd(tick.last_price).expect("should convert price");
    let price = PriceOfOneBTC::new(usd_cents);
    assert!(price.sats_to_cents_round_down(Satoshis::from(100_000_000)) > UsdCents::from(0));
}

#[tokio::test]
async fn update_price() -> anyhow::Result<()> {
    let pool = init_pool().await?;

    let outbox = Outbox::<DummyEvent>::init(&pool, obix::MailboxConfig::builder().build()?).await?;

    let price = Price::new(&outbox);

    let initial_price_cents = rand::rng().random_range(1_000_000..10_000_000);
    let initial_price = PriceOfOneBTC::new(UsdCents::from(initial_price_cents));
    publish_dummy_price_event(&outbox, initial_price).await?;

    let first_observed_price = wait_for_price_to_be_updated(&price, initial_price).await?;
    assert_eq!(first_observed_price, initial_price);

    let updated_expected_price_cents = rand::rng().random_range(1_000_000..10_000_000);
    let updated_expected_price = PriceOfOneBTC::new(UsdCents::from(updated_expected_price_cents));
    publish_dummy_price_event(&outbox, updated_expected_price).await?;

    let second_observed_price =
        wait_for_price_to_be_updated(&price, updated_expected_price).await?;
    assert_eq!(second_observed_price, updated_expected_price);

    Ok(())
}

#[test]
fn cents_to_sats_trivial() {
    let price = PriceOfOneBTC::new(UsdCents::try_from_usd(dec!(1000)).unwrap());
    let cents = UsdCents::try_from_usd(dec!(1000)).unwrap();
    assert_eq!(
        Satoshis::try_from_btc(dec!(1)).unwrap(),
        price.cents_to_sats_round_up(cents)
    );
}

#[test]
fn cents_to_sats_complex() {
    let price = PriceOfOneBTC::new(UsdCents::try_from_usd(dec!(60000)).unwrap());
    let cents = UsdCents::try_from_usd(dec!(100)).unwrap();
    assert_eq!(
        Satoshis::try_from_btc(dec!(0.00166667)).unwrap(),
        price.cents_to_sats_round_up(cents)
    );
}

#[test]
fn sats_to_cents_trivial() {
    let price = PriceOfOneBTC::new(UsdCents::from(5_000_000));
    let sats = Satoshis::from(10_000);
    assert_eq!(UsdCents::from(500), price.sats_to_cents_round_down(sats));
}

#[test]
fn sats_to_cents_complex() {
    let price = PriceOfOneBTC::new(UsdCents::from(5_000_000));
    let sats = Satoshis::from(12_345);
    assert_eq!(UsdCents::from(617), price.sats_to_cents_round_down(sats));
}
