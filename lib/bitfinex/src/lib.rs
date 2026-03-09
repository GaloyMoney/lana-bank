#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;
mod wire;

use chrono::Utc;
use hmac::{Hmac, Mac as _};
use reqwest::{Client, Url};
use serde_json::json;
use sha2::Sha384;
use tracing_macros::record_error_severity;

pub use config::{BitfinexConfig, BitfinexDirectoryConfig};
pub use error::*;
pub use wire::*;

#[derive(Clone)]
pub struct BitfinexClient {
    http_client: Client,
    api_key: String,
    api_secret: Vec<u8>,
    endpoint: Url,
    is_test: bool,
}

impl std::fmt::Debug for BitfinexClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitfinexClient")
            .field("endpoint", &self.endpoint)
            .field("is_test", &self.is_test)
            .field("api_key", &"<redacted>")
            .field("api_secret", &"<redacted>")
            .finish()
    }
}

impl BitfinexClient {
    pub fn try_new(
        config: BitfinexConfig,
        directory_config: BitfinexDirectoryConfig,
    ) -> Result<Self, BitfinexError> {
        let endpoint = if config.bitfinex_test {
            directory_config.testing_url
        } else {
            directory_config.production_url
        };

        let _ = rustls::crypto::ring::default_provider().install_default();
        Ok(Self {
            http_client: Client::new(),
            api_key: config.api_key,
            api_secret: config.api_secret.into_bytes(),
            endpoint,
            is_test: config.bitfinex_test,
        })
    }

    pub fn is_testnet(&self) -> bool {
        self.is_test
    }

    #[record_error_severity]
    #[tracing::instrument(name = "bitfinex.list_wallets", skip(self))]
    pub async fn list_wallets(&self) -> Result<Vec<WalletEntry>, BitfinexError> {
        let response = self
            .authenticated_post("v2/auth/r/wallets", json!({}))
            .await?;

        let arr = response.as_array().ok_or_else(|| {
            BitfinexError::UnexpectedResponseFormat(
                "list_wallets response is not an array".to_string(),
            )
        })?;

        arr.iter().map(WalletEntry::from_value).collect()
    }

    #[record_error_severity]
    #[tracing::instrument(name = "bitfinex.get_deposit_address", skip(self))]
    pub async fn get_deposit_address(&self, renew: bool) -> Result<String, BitfinexError> {
        let op_renew: i32 = if renew { 1 } else { 0 };
        let body = json!({
            "wallet": "exchange",
            "method": "bitcoin",
            "op_renew": op_renew,
        });

        let response = self
            .authenticated_post("v2/auth/w/deposit/address", body)
            .await?;

        let deposit = DepositAddressResponse::from_value(&response)?;
        Ok(deposit.address)
    }

    async fn authenticated_post(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, BitfinexError> {
        let nonce = Utc::now().timestamp_millis().to_string();
        let body_json = serde_json::to_string(&body)?;

        let signature_payload = format!("/api/{path}{nonce}{body_json}");

        let mut mac = Hmac::<Sha384>::new_from_slice(&self.api_secret)
            .expect("HMAC can take key of any size");
        mac.update(signature_payload.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        let url = self.endpoint.join(path).expect("valid URL");

        let response = self
            .http_client
            .post(url)
            .header("bfx-apikey", &self.api_key)
            .header("bfx-nonce", &nonce)
            .header("bfx-signature", &signature)
            .header("content-type", "application/json")
            .body(body_json)
            .send()
            .await?;

        let value: serde_json::Value = response.json().await?;

        // Check for error response: ["error", code, message]
        if let Some(arr) = value.as_array() {
            if arr.len() >= 3 && arr[0].as_str() == Some("error") {
                let message = arr[2].as_str().unwrap_or("unknown error").to_string();
                return Err(BitfinexError::BitfinexApiError { message });
            }
        }

        Ok(value)
    }
}
