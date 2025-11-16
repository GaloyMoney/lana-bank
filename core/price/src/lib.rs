#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
mod price_client;
mod primitives;

use cached::proc_macro::cached;

use std::{sync::Arc, time::Duration};

use core_money::UsdCents;

use error::PriceError;
use price_client::{PriceClient, PriceTick};
pub use primitives::*;

#[derive(Clone)]
pub struct Price {
    price_client: Arc<dyn PriceClient>,
}

impl Price {
    pub fn new() -> Self {
        Self {
            price_client: Arc::new(price_client::default_client()),
        }
    }

    pub fn new_with_client<C>(price_client: C) -> Self
    where
        C: PriceClient + 'static,
    {
        Self {
            price_client: Arc::new(price_client),
        }
    }

    pub async fn usd_cents_per_btc(&self) -> Result<PriceOfOneBTC, PriceError> {
        usd_cents_per_btc_cached(self.price_client.as_ref()).await
    }
}

impl Default for Price {
    fn default() -> Self {
        Self::new()
    }
}

#[cached(time = 60, result = true, key = "()", convert = r#"{}"#)]
async fn usd_cents_per_btc_cached(
    price_client: &dyn PriceClient,
) -> Result<PriceOfOneBTC, PriceError> {
    if std::env::var("BFX_LOCAL_PRICE").is_ok() {
        return Ok(PriceOfOneBTC::new(UsdCents::try_from_usd(
            rust_decimal_macros::dec!(100_000),
        )?));
    }

    let PriceTick { last_price } = price_client.btc_usd_tick().await?;
    Ok(PriceOfOneBTC::new(UsdCents::try_from_usd(last_price)?))
}
