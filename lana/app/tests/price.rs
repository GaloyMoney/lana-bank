mod helpers;
use lana_app::{
    job::{JobExecutorConfig, Jobs},
    price::Price,
};

#[tokio::test]
async fn get_price() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let jobs = Jobs::new(&pool, JobExecutorConfig::default());
    let price_service = Price::init(&jobs).await?;
    let res = price_service.usd_cents_per_btc().await;
    assert!(res.is_ok());

    Ok(())
}
