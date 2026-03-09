mod auth;
mod config;
pub mod error;
pub mod response;

use reqwest::Client as ReqwestClient;
use tracing_macros::record_error_severity;

pub use auth::BfxAuthClient;
pub use config::{BfxAuthConfig, BfxDirectoryConfig};
pub use error::BfxClientError;
pub use response::{BfxErrorResponse, BfxNotification, BtcUsdTick, DepositAddress, Wallet};

const BASE_URL: &str = "https://api-pub.bitfinex.com/v2/";

#[derive(Clone, Default)]
pub struct BfxClient {
    client: ReqwestClient,
}

impl BfxClient {
    pub fn new() -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        BfxClient {
            client: ReqwestClient::builder()
                .use_rustls_tls()
                .build()
                .expect("should always build BfxClient"),
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "bfx.btc_usd_tick", skip(self), fields(url, response))]
    pub async fn btc_usd_tick(&self) -> Result<BtcUsdTick, BfxClientError> {
        let url = format!("{BASE_URL}ticker/tBTCUSD");
        tracing::Span::current().record("url", tracing::field::display(&url));

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await?;
        let tick = extract_response_data::<BtcUsdTick>(response).await?;

        tracing::Span::current().record("response", tracing::field::debug(&tick));

        Ok(tick)
    }
}

async fn extract_response_data<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, BfxClientError> {
    let status = response.status();
    let response_text = response.text().await?;

    if let Some(error) = extract_error_response(&response_text) {
        return Err(error);
    }

    if status.is_success() {
        Ok(serde_json::from_str::<T>(&response_text)?)
    } else {
        Err(BfxClientError::UnexpectedHttpStatus {
            status,
            body: response_text,
        })
    }
}

fn extract_error_response(response_text: &str) -> Option<BfxClientError> {
    if let Ok(data) = serde_json::from_str::<BfxErrorResponse>(response_text) {
        return Some(BfxClientError::from((
            data.event,
            data.code,
            data.description,
        )));
    }

    if let Ok(response::BfxAuthErrorResponse(event, code, desc)) =
        serde_json::from_str::<response::BfxAuthErrorResponse>(response_text)
        && event == "error"
    {
        return Some(BfxClientError::from_auth_error(code, desc));
    }

    None
}
