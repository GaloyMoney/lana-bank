mod config;
mod error;
mod response;

use reqwest::{Client, Url};
use serde_json::{json, Value};

pub use config::BitgoConfig;
pub use error::*;
pub use response::*;

#[derive(Debug, Clone)]
pub struct BitgoClient {
    http_client: Client,
    long_lived_token: String,
    endpoint: Url,
    passphrase: String,
    enterprise_id: String,
    coin: String,
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
        })
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
}
