pub mod error;

use crate::bfx_client::BfxClient;
use crate::sources::error::PriceClientError;
use crate::{PriceError, PriceOfOneBTC};
use core_money::UsdCents;

use async_trait::async_trait;

#[async_trait]
pub trait PriceClient: Send + Sync {
    async fn fetch_btc_usd_price(&self) -> Result<PriceOfOneBTC, PriceError>;
}

#[async_trait]
impl PriceClient for BfxClient {
    async fn fetch_btc_usd_price(&self) -> Result<PriceOfOneBTC, PriceError> {
        let tick = self.btc_usd_tick().await.map_err(PriceClientError::from)?;
        let price = PriceOfOneBTC::new(UsdCents::try_from_usd(tick.last_price)?);
        Ok(price)
    }
}
