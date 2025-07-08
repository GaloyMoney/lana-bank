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

        Ok(serde_json::from_slice::<Notification>(payload).unwrap())
    }

    #[tracing::instrument(name = "bitgo.generate_wallet", skip(self), err)]
    pub async fn generate_wallet(&self, label: &str) -> Result<(Wallet, Value), BitgoError> {
        let url = self
            .endpoint
            .join(&self.coin)
            .expect("correct URL")
            .join("wallet/generate")
            .expect("correct URL");

        let request = self
            .http_client
            .post(url)
            .bearer_auth(&self.long_lived_token)
            .json(&json!({
                "label": label,
                "passphrase": &self.passphrase,
                "enterprise": &self.enterprise_id
            }));

        let response: Value = request.send().await?.json().await?;
        let wallet = serde_json::from_value(response.clone()).unwrap();

        Ok((wallet, response))
    }

    #[tracing::instrument(name = "bitgo.get_wallet", skip(self), err)]
    pub async fn get_wallet(&self, id: &str) -> Result<(Wallet, Value), BitgoError> {
        let url = self
            .endpoint
            .join(&format!("wallet/{id}"))
            .expect("valid URL");

        let request = self
            .http_client
            .get(url)
            .bearer_auth(&self.long_lived_token);

        let response: Value = request.send().await?.json().await?;
        let wallet = serde_json::from_value(response.clone()).unwrap();

        Ok((wallet, response))
    }

    pub async fn get_transfer(&self, id: &str, wallet: &str) -> Result<Transfer, BitgoError> {
        let url = self
            .endpoint
            .join(&format!("wallet/{wallet}/transfer/{id}"))
            .expect("valid URL");

        let request = self
            .http_client
            .get(url)
            .bearer_auth(&self.long_lived_token);

        Ok(request.send().await?.json().await?)
    }
}
