#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod error;
mod response;

use reqwest::Client as ReqwestClient;
use tracing::instrument;

pub use error::BfxClientError;
use response::*;

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

    #[instrument(name = "bitfinex.btc_usd_tick", skip(self), err)]
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

    #[instrument(name = "bitfinex.extract_response_data", skip(response), err)]
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
