mod bfx_client;
pub mod error;
use cached::proc_macro::cached;
mod job;

use crate::{
    data_export::Export,
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
    pub async fn init(jobs: &Jobs, export: &Export) -> Result<Self, PriceError> {
        let price = Self {
            bfx: BfxClient::new(),
            _jobs: jobs.clone(),
        };

        jobs.add_initializer_and_spawn_unique(
            job::ExportPriceInitializer::new(&price, export),
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
    let last_price = bfx.btc_usd_tick().await?.last_price;
    Ok(PriceOfOneBTC::new(UsdCents::try_from_usd(last_price)?))
}
