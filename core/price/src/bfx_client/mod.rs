pub mod error;
mod response;

use reqwest::Client as ReqwestClient;

use core_money::UsdCents;

use crate::PriceOfOneBTC;
use error::BfxClientError;
use response::{BfxErrorResponse, BtcUsdTick};
use tracing::instrument;
use tracing_macros::record_error_severity;

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

#[record_error_severity]
#[instrument(name = "core.price.bfx_client.fetch_price", skip(client))]
pub async fn fetch_price(
    client: std::sync::Arc<BfxClient>,
) -> Result<PriceOfOneBTC, BfxClientError> {
    let tick = client.btc_usd_tick().await?;
    let usd_cents =
        UsdCents::try_from_usd(tick.last_price).map_err(BfxClientError::ConversionError)?;
    Ok(PriceOfOneBTC::new(usd_cents))
}
