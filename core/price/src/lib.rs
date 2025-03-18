mod bfx_client;
pub mod error;
mod primitives;

use cached::proc_macro::cached;

use core_money::UsdCents;

use bfx_client::BfxClient;
use error::PriceError;
pub use primitives::*;

#[derive(Clone)]
pub struct Price {
    bfx: BfxClient,
}

impl Price {
    pub fn new() -> Self {
        Self {
            bfx: BfxClient::new(),
        }
    }

    pub async fn usd_cents_per_btc(&self) -> Result<PriceOfOneBTC, PriceError> {
        usd_cents_per_btc_cached(&self.bfx).await
    }

    pub async fn avg_btc_price_in_24_hours(&self) -> Result<PriceOfOneBTC, PriceError> {
        avg_btc_price_in_24_hours_cached(&self.bfx).await
    }
}

impl Default for Price {
    fn default() -> Self {
        Self::new()
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

#[cached(time = 86400, result = true, key = "()", convert = r#"{}"#)]
async fn avg_btc_price_in_24_hours_cached(bfx: &BfxClient) -> Result<PriceOfOneBTC, PriceError> {
    use rust_decimal::Decimal;

    if std::env::var("BFX_LOCAL_PRICE").is_ok() {
        return Ok(PriceOfOneBTC::new(UsdCents::try_from_usd(
            rust_decimal_macros::dec!(100_000),
        )?));
    }

    let btc_usd_hist_ticks = bfx.btc_usd_hist_ticks().await?;

    let avg_price = PriceOfOneBTC::new(UsdCents::try_from_usd(
        (btc_usd_hist_ticks
            .iter()
            .map(|tick| (tick.bid + tick.ask))
            .sum::<Decimal>()
            / Decimal::TWO
            / Decimal::from(btc_usd_hist_ticks.len()))
        .ceil(),
    )?);

    Ok(avg_price)
}
