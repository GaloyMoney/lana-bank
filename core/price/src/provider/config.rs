use serde::{Deserialize, Serialize};

use super::error::PriceProviderError;
use crate::PriceOfOneBTC;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PriceProviderConfig {
    Bitfinex,
}

impl PriceProviderConfig {
    pub async fn fetch_price(&self) -> Result<PriceOfOneBTC, PriceProviderError> {
        match self {
            PriceProviderConfig::Bitfinex => {
                let client = bfx_client::BfxClient::new();
                let tick = client.btc_usd_tick().await?;
                let usd_cents = money::UsdCents::try_from_usd(tick.last_price).map_err(|e| {
                    PriceProviderError::BfxClientError(bfx_client::BfxClientError::ConversionError(
                        e,
                    ))
                })?;
                Ok(PriceOfOneBTC::new(usd_cents))
            }
        }
    }
}
