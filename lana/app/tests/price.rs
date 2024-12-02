mod helpers;
use lana_app::{
    data_export::Export,
    job::{JobExecutorConfig, Jobs},
    price::Price,
};

#[tokio::test]
async fn get_price() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let jobs = Jobs::new(&pool, JobExecutorConfig::default());
    let export = Export::new("".to_string(), &jobs);
    let price_service = Price::init(&jobs, &export).await?;
    let res = price_service.usd_cents_per_btc().await;
    assert!(res.is_ok());

    Ok(())
}
