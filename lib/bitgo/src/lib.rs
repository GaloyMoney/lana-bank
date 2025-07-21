#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;
mod wire;

use hmac::{Hmac, Mac as _};
use reqwest::{header::HeaderMap, Client, Url};
use serde_json::{json, Value};
use sha2::Sha256;

pub use config::BitgoConfig;
pub use error::*;
pub use wire::*;

#[derive(Debug, Clone)]
pub struct BitgoClient {
    http_client: Client,
    long_lived_token: String,
    endpoint: Url,
    passphrase: String,
    enterprise_id: String,
    coin: String,
    webhook_secret: Vec<u8>,
}

impl BitgoClient {
    pub fn new(config: BitgoConfig) -> Result<Self, BitgoError> {
        let coin = if config.bitgo_test { "tbtc4" } else { "btc" };
        let endpoint = config
            .express_endpoint
            .parse()
            .map_err(|_| BitgoError::InvalidEndpoint(config.express_endpoint))?;

        Ok(Self {
            http_client: Client::new(),
            long_lived_token: config.long_lived_token,
            endpoint,
            passphrase: config.passphrase,
            enterprise_id: config.enterprise_id,
            coin: coin.to_owned(),
            webhook_secret: config.webhook_secret,
        })
    }

    pub fn validate_webhook_notification(
        &self,
        headers: &HeaderMap,
        payload: &[u8],
    ) -> Result<Notification, BitgoError> {
        let signature = headers
            .get("x-signature-sha256")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| hex::decode(s).ok())
            .ok_or(BitgoError::MissingWebhookSignature)?;

        let mut mac =
            Hmac::<Sha256>::new_from_slice(&self.webhook_secret).expect("valid length of secret");
        mac.update(payload);
        mac.verify_slice(&signature)?;

        Ok(serde_json::from_slice::<Notification>(payload)?)
    }

    #[tracing::instrument(name = "bitgo.generate_wallet", skip(self), fields(response, url), err)]
    pub async fn generate_wallet(&self, label: &str) -> Result<(Wallet, Value), BitgoError> {
        let url = self.url_with_coin("wallet/generate");

        tracing::Span::current().record("url", tracing::field::display(&url));

        let request = self
            .http_client
            .post(url)
            .bearer_auth(&self.long_lived_token)
            .json(&json!({
                "label": label,
                "passphrase": &self.passphrase,
                "enterprise": &self.enterprise_id
            }));

        let response_json: Value = request.send().await?.json().await?;

        tracing::Span::current().record("response", tracing::field::display(&response_json));

        let response: GenerateWalletResponse = serde_json::from_value(response_json.clone())?;

        Ok((response.wallet, response_json))
    }

    #[tracing::instrument(name = "bitgo.get_wallet", skip(self), fields(response, url), err)]
    pub async fn get_wallet(&self, id: &str) -> Result<(Wallet, Value), BitgoError> {
        let url = self.url(&format!("wallet/{id}"));

        tracing::Span::current().record("url", tracing::field::display(&url));

        let request = self
            .http_client
            .get(url)
            .bearer_auth(&self.long_lived_token);

        let response: Value = request.send().await?.json().await?;

        tracing::Span::current().record("response", tracing::field::display(&response));

        let wallet = serde_json::from_value(response.clone())?;

        Ok((wallet, response))
    }

    #[tracing::instrument(name = "bitgo.get_transfer", skip(self), fields(response, url), err)]
    pub async fn get_transfer(&self, id: &str, wallet: &str) -> Result<Transfer, BitgoError> {
        let url = self.url(&format!("wallet/{wallet}/transfer/{id}"));

        tracing::Span::current().record("url", tracing::field::display(&url));

        let request = self
            .http_client
            .get(url)
            .bearer_auth(&self.long_lived_token);

        let response: Value = request.send().await?.json().await?;

        tracing::Span::current().record("response", tracing::field::display(&response));

        Ok(serde_json::from_value(response)?)
    }
}

impl BitgoClient {
    fn url(&self, path: &str) -> Url {
        self.endpoint
            .join("api/v2/")
            .expect("valid URL")
            .join(path)
            .expect("valid URL")
    }

    fn url_with_coin(&self, path: &str) -> Url {
        self.endpoint
            .join(&format!("api/v2/{}/", self.coin))
            .expect("valid URL")
            .join(path)
            .expect("valid URL")
    }
}
