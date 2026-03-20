mod helpers;

use rand::RngExt;
use rust_decimal_macros::dec;

use chrono::Utc;
use core_price::{ExchangeRateMetadata, ExchangeRateType, Price, PriceOfOneBTC};
use helpers::{DummyEvent, init_pool, publish_dummy_price_event, wait_for_price_to_be_updated};
use money::{Amount as MoneyAmount, CurrencyCode, Satoshis, UsdCents};
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

#[tokio::test]
async fn exchange_rate_metadata_returns_identity_for_same_currency() -> anyhow::Result<()> {
    let pool = init_pool().await?;

    let outbox = Outbox::<DummyEvent>::init(&pool, obix::MailboxConfig::builder().build()?).await?;

    let price = Price::new(&outbox);
    let quote_amount = MoneyAmount::from(UsdCents::from(70_000));

    let requested_at = Utc::now();
    let observed = price
        .exchange_rate_metadata(
            ExchangeRateType::Spot,
            CurrencyCode::USD,
            CurrencyCode::USD,
            quote_amount,
        )
        .await?;
    let completed_at = Utc::now();

    assert_eq!(observed.base_currency, CurrencyCode::USD);
    assert_eq!(observed.quote_currency, CurrencyCode::USD);
    assert_eq!(observed.rate_type, ExchangeRateType::Spot);
    assert_eq!(observed.reference_rate, rust_decimal::Decimal::ONE);
    assert!(observed.exchange_rate_timestamp >= requested_at);
    assert!(observed.exchange_rate_timestamp <= completed_at);
    assert_eq!(observed.base_currency_value, quote_amount);

    Ok(())
}

#[tokio::test]
async fn exchange_rate_metadata_converts_btc_to_usd() -> anyhow::Result<()> {
    let pool = init_pool().await?;

    let outbox = Outbox::<DummyEvent>::init(&pool, obix::MailboxConfig::builder().build()?).await?;

    let price = Price::new(&outbox);
    let exchange_rate = PriceOfOneBTC::new(UsdCents::from(7_000_000));
    let quote_amount = MoneyAmount::from(Satoshis::from(100_000_000));

    publish_dummy_price_event(&outbox, exchange_rate).await?;
    let _ = wait_for_price_to_be_updated(&price, exchange_rate).await?;

    let requested_at = Utc::now();
    let observed = price
        .exchange_rate_metadata(
            ExchangeRateType::Spot,
            CurrencyCode::USD,
            CurrencyCode::BTC,
            quote_amount,
        )
        .await?;
    let completed_at = Utc::now();

    assert_eq!(
        observed,
        ExchangeRateMetadata {
            base_currency: CurrencyCode::USD,
            quote_currency: CurrencyCode::BTC,
            rate_type: ExchangeRateType::Spot,
            reference_rate: dec!(70000),
            exchange_rate_timestamp: observed.exchange_rate_timestamp,
            base_currency_value: MoneyAmount::from(UsdCents::from(7_000_000)),
        }
    );
    assert!(observed.exchange_rate_timestamp >= requested_at);
    assert!(observed.exchange_rate_timestamp <= completed_at);

    Ok(())
}

#[tokio::test]
async fn exchange_rate_metadata_rejects_unsupported_rate_type_for_conversion() -> anyhow::Result<()>
{
    let pool = init_pool().await?;

    let outbox = Outbox::<DummyEvent>::init(&pool, obix::MailboxConfig::builder().build()?).await?;

    let price = Price::new(&outbox);
    let exchange_rate = PriceOfOneBTC::new(UsdCents::from(7_000_000));
    let quote_amount = MoneyAmount::from(Satoshis::from(100_000_000));

    publish_dummy_price_event(&outbox, exchange_rate).await?;
    let _ = wait_for_price_to_be_updated(&price, exchange_rate).await?;

    let err = price
        .exchange_rate_metadata(
            ExchangeRateType::Close,
            CurrencyCode::USD,
            CurrencyCode::BTC,
            quote_amount,
        )
        .await
        .expect_err("close rates are not yet implemented");

    assert!(matches!(
        err,
        core_price::error::PriceError::UnsupportedExchangeRateType {
            rate_type: ExchangeRateType::Close
        }
    ));

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
