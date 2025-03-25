pub mod error;
mod response;

use reqwest::Client as ReqwestClient;

use error::BfxClientError;
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

    pub async fn btc_usd_tick(&self) -> Result<BtcUsdTick, BfxClientError> {
        let url = format!("{}ticker/tBTCUSD", BASE_URL);
        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await?;
        let tick = Self::extract_response_data::<BtcUsdTick>(response).await?;

        Ok(tick)
    }

    pub async fn btc_usd_hist_ticks(&self) -> Result<Vec<BtcUsdHistTick>, BfxClientError> {
        let url = format!("{}tickers/hist?symbols=tBTCUSD&limit=24", BASE_URL);
        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await?;
        let btc_usd_hist_ticks =
            Self::extract_response_data_array::<BtcUsdHistTick>(response).await?;

        Ok(btc_usd_hist_ticks)
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

    async fn extract_response_data_array<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<Vec<T>, BfxClientError> {
        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            Ok(serde_json::from_str::<Vec<T>>(&response_text)?)
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
