use std::time::Duration;
use std::{sync::Arc, time::Instant};

use base64::{prelude::BASE64_STANDARD, Engine};
use p256::ecdsa::{signature::Signer as _, Signature, SigningKey};
use p256::pkcs8::DecodePrivateKey as _;
use p256::SecretKey;
use reqwest::{Client, Method, Proxy, RequestBuilder, Url};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use tokio::sync::Mutex;

mod config;
mod error;
mod wire;

pub use config::{KomainuConfig, KomainuProxy, KomainuSecretKey};
pub use error::KomainuError;
pub use wire::*;

#[derive(Clone)]
pub struct KomainuClient {
    http_client: Client,
    access_token: Arc<Mutex<Option<AccessToken>>>,
    signing_key: SigningKey,
    config: KomainuConfig,
}

impl KomainuClient {
    pub fn new(config: KomainuConfig) -> Self {
        let signing_key = match &config.secret_key {
            KomainuSecretKey::Encrypted { dem, passphrase } => {
                SecretKey::from_pkcs8_encrypted_pem(dem, passphrase)
                    .expect("valid passphrase")
                    .into()
            }
            KomainuSecretKey::Plain { dem } => SecretKey::from_pkcs8_pem(dem).unwrap().into(),
        };

        let http_client = if let Some(KomainuProxy::Socks5(proxy)) = &config.proxy {
            Client::builder()
                .proxy(Proxy::all(format!("socks5://{proxy}")).expect("correct proxy scheme"))
                .build()
                .expect("correct client")
        } else {
            Client::new()
        };

        Self {
            http_client,
            access_token: Default::default(),
            signing_key,
            config,
        }
    }

    pub async fn list_wallets(&self) -> Result<Value, KomainuError> {
        Ok(self.get("v1/custody/wallets").await?)
    }
}

impl KomainuClient {
    async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, reqwest::Error> {
        self.request::<()>(Method::GET, endpoint, None)
            .await?
            .send()
            .await?
            .json()
            .await
    }

    async fn request<T: Serialize>(
        &self,
        method: Method,
        endpoint: &str,
        _body: Option<T>,
    ) -> Result<RequestBuilder, reqwest::Error> {
        let url: Url = format!("https://api-uat.komainu.io/{endpoint}")
            .parse()
            .expect("valid URL");

        let access_token = self.get_access_token().await?;
        let timestamp = chrono::Utc::now().timestamp_millis();

        let canonical_string = format!(
            "{},{},{},sha256={},sha256={},{}",
            url.host_str().expect("URL with host"),
            method.as_str().to_lowercase(),
            url.path(),
            BASE64_STANDARD.encode(Sha256::digest("")),
            BASE64_STANDARD.encode(Sha256::digest(&access_token)),
            timestamp
        );

        let signature: Signature = self.signing_key.sign(canonical_string.as_bytes());

        Ok(self
            .http_client
            .get(url)
            .bearer_auth(access_token)
            .header("X-Timestamp", timestamp)
            .header(
                "X-Signature",
                BASE64_STANDARD.encode(signature.to_der().to_bytes()),
            ))
    }

    async fn get_access_token(&self) -> Result<String, reqwest::Error> {
        let mut access_token = self.access_token.lock().await;
        match access_token.as_ref() {
            Some(token) if token.expires_at > Instant::now() => Ok(token.access_token.clone()),
            _ => {
                let new_token = self.refresh_token().await?;
                let token = new_token.access_token.clone();
                *access_token = Some(new_token);
                Ok(token)
            }
        }
    }

    async fn refresh_token(&self) -> Result<AccessToken, reqwest::Error> {
        let response: GetTokenResponse = self
            .http_client
            .post("https://api-uat.komainu.io/v1/auth/token")
            .json(&GetToken {
                api_user: &self.config.api_user,
                api_secret: &self.config.api_secret,
            })
            .send()
            .await?
            .json()
            .await?;

        Ok(AccessToken {
            access_token: response.access_token,
            expires_at: Instant::now() + Duration::from_secs(response.expires_in),
        })
    }
}

#[derive(Clone)]
struct AccessToken {
    access_token: String,
    expires_at: Instant,
}
