use core_price::Price;

#[tokio::test]
async fn avg_daily_price() -> anyhow::Result<()> {
    let price = Price::new();
    let avg_daily_price = price.avg_btc_price_in_24_hours().await;
    assert!(avg_daily_price.is_ok());

    Ok(())
}
