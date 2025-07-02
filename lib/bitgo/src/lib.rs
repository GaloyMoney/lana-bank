mod error;
mod response;

use reqwest::{Client, Url};
use serde_json::{json, Value};

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
    pub async fn init() -> Result<Self, BitgoError> {
        todo!()
    }

    pub async fn generate_wallet(
        &self,
        label: impl AsRef<str>,
    ) -> Result<(Wallet, Value), BitgoError> {
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
                "label": label.as_ref(),
                "passphrase": &self.passphrase,
                "enterprise": &self.enterprise_id
            }));

        let response: Value = request.send().await?.json().await?;
        let wallet = serde_json::from_value(response.clone()).unwrap();

        Ok((wallet, response))
    }

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
