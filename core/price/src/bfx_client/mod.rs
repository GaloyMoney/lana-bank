pub mod error;
mod response;

use reqwest::Client as ReqwestClient;

use error::BfxClientError;
use response::{BfxErrorResponse, BtcUsdTick};

const BASE_URL: &str = "https://api-pub.bitfinex.com/v2/";

#[derive(Clone, Default)]
pub struct BfxClient {
    client: ReqwestClient,
}

impl BfxClient {
    pub fn new() -> Self {
        BfxClient {
            client: ReqwestClient::builder()
                .use_rustls_tls()
                .build()
                .expect("should always build BfxClient"),
        }
    }

    pub async fn btc_usd_tick(&self) -> Result<BtcUsdTick, BfxClientError> {
        let url = format!("{BASE_URL}ticker/tBTCUSD");
        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await?;
        let tick = Self::extract_response_data::<BtcUsdTick>(response).await?;

        Ok(tick)
    }

    async fn extract_response_data<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, BfxClientError> {
        let status = response.status();
        let response_text = response.text().await?;
        if status.is_success() {
            Ok(serde_json::from_str::<T>(&response_text)?)
        } else {
            let data = serde_json::from_str::<BfxErrorResponse>(&response_text)?;
            Err(BfxClientError::from((
                data.event,
                data.code,
                data.description,
            )))
        }
    }
}

use crate::PriceError;
use crate::PriceOfOneBTC;
use crate::UsdCents;
pub async fn usd_cents_per_btc_cached(bfx: &BfxClient) -> Result<PriceOfOneBTC, PriceError> {
    if std::env::var("BFX_LOCAL_PRICE").is_ok() {
        return Ok(PriceOfOneBTC::new(UsdCents::try_from_usd(
            rust_decimal_macros::dec!(100_000),
        )?));
    }

    let last_price = bfx.btc_usd_tick().await?.last_price;
    Ok(PriceOfOneBTC::new(UsdCents::try_from_usd(last_price)?))
}
