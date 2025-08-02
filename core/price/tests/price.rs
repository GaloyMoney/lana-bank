use core_price::{Price, PriceOfOneBTC};
use core_money::{Satoshis, UsdCents};
use rust_decimal_macros::dec;

#[tokio::test]
async fn get_price() -> anyhow::Result<()> {
    let price = Price::new();
    let res = price.usd_cents_per_btc().await;
    assert!(res.is_ok());

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
