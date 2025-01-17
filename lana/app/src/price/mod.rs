mod bfx_client;
pub mod error;
use cached::proc_macro::cached;
mod job;

use crate::{
    job::Jobs,
    primitives::{PriceOfOneBTC, UsdCents},
};

use bfx_client::BfxClient;
use error::PriceError;

#[derive(Clone)]
pub struct Price {
    bfx: BfxClient,
    _jobs: Jobs,
}

impl Price {
    pub async fn init(jobs: &Jobs) -> Result<Self, PriceError> {
        let price = Self {
            bfx: BfxClient::new(),
            _jobs: jobs.clone(),
        };

        jobs.add_initializer_and_spawn_unique(
            job::ExportPriceInitializer::new(&price),
            job::ExportPriceJobConfig::default(),
        )
        .await?;
        Ok(price)
    }

    pub async fn usd_cents_per_btc(&self) -> Result<PriceOfOneBTC, PriceError> {
        usd_cents_per_btc_cached(&self.bfx).await
    }
}

#[cached(time = 60, result = true, key = "()", convert = r#"{}"#)]
async fn usd_cents_per_btc_cached(bfx: &BfxClient) -> Result<PriceOfOneBTC, PriceError> {
    if std::env::var("BFX_LOCAL_PRICE").is_ok() {
        return Ok(PriceOfOneBTC::new(UsdCents::try_from_usd(
            rust_decimal_macros::dec!(100_000),
        )?));
    }

    let last_price = bfx.btc_usd_tick().await?.last_price;
    Ok(PriceOfOneBTC::new(UsdCents::try_from_usd(last_price)?))
}
