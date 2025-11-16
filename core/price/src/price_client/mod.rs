pub mod error;

use async_trait::async_trait;
use rust_decimal::Decimal;

use self::error::PriceClientError;

#[derive(Debug, Clone)]
pub struct PriceTick {
    pub last_price: Decimal,
}

#[async_trait]
pub trait PriceClient: Send + Sync {
    async fn btc_usd_tick(&self) -> Result<PriceTick, PriceClientError>;
}

pub fn default_client() -> impl PriceClient {
    bitfinex::BfxClient::new()
}

#[async_trait]
impl PriceClient for bitfinex::BfxClient {
    async fn btc_usd_tick(&self) -> Result<PriceTick, PriceClientError> {
        let tick = <bitfinex::BfxClient>::btc_usd_tick(self).await?;
        Ok(PriceTick {
            last_price: tick.last_price,
        })
    }
}
