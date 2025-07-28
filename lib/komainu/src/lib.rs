#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;
mod wire;

use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::{Engine as _, prelude::BASE64_STANDARD};
use hmac::{Hmac, Mac as _};
use p256::{
    SecretKey,
    ecdsa::{Signature, SigningKey, signature::Signer as _},
    pkcs8::DecodePrivateKey as _,
};
use reqwest::header::HeaderMap;
use reqwest::{
    Client, Method, RequestBuilder, Url,
    header::{CONTENT_TYPE, HeaderValue},
};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest as _, Sha256};
use tokio::sync::Mutex;

pub use config::{KomainuConfig, KomainuDirectoryConfig, KomainuSecretKey};
pub use error::KomainuError;
pub use wire::{EntityType, EventType, Notification, Request, Transaction, Wallet};
use wire::{Fallible, GetToken, GetTokenResponse, Many};

#[derive(Clone)]
pub struct KomainuClient {
    http_client: Client,
    access_token: Arc<Mutex<Option<AccessToken>>>,
    signing_key: SigningKey,
    endpoint: Url,
    get_token_request: GetToken,
    webhook_secret: Vec<u8>,
}

impl KomainuClient {
    pub fn try_new(
        config: KomainuConfig,
        directory_config: KomainuDirectoryConfig,
    ) -> Result<Self, KomainuError> {
        let signing_key = match &config.secret_key {
            KomainuSecretKey::Encrypted { dem, passphrase } => {
                SecretKey::from_pkcs8_encrypted_pem(dem, passphrase)
                    .map_err(|_| KomainuError::SecretKey)?
                    .into()
            }
            KomainuSecretKey::Plain { dem } => SecretKey::from_pkcs8_pem(dem)
                .map_err(|_| KomainuError::SecretKey)?
                .into(),
        };

        let get_token_request = GetToken {
            api_user: config.api_user.clone(),
            api_secret: config.api_secret.clone(),
        };

        let endpoint = if config.komainu_test {
            directory_config.testing_url
        } else {
            directory_config.production_url
        };

        Ok(Self {
            http_client: Client::new(),
            access_token: Default::default(),
            signing_key,
            get_token_request,
            endpoint,
            webhook_secret: config.webhook_secret,
        })
    }

    pub fn validate_webhook_notification(
        &self,
        headers: &HeaderMap,
        payload: &[u8],
    ) -> Result<Notification, KomainuError> {
        // https://docs.komainu.io/apispec/#section/Notifications/Webhook-validation

        let signature = headers
            .get("x-komainu-signature")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| hex::decode(s).ok())
            .ok_or(KomainuError::MissingWebhookHeaders)?;

        let timestamp = headers
            .get("x-komainu-timestamp")
            .and_then(|v| v.to_str().ok())
            .ok_or(KomainuError::MissingWebhookHeaders)?;

        let mut mac =
            Hmac::<Sha256>::new_from_slice(&self.webhook_secret).expect("valid length of secret");
        mac.update(timestamp.as_bytes());
        mac.update(b".");
        mac.update(payload);
        mac.verify_slice(&signature)?;

        Ok(serde_json::from_slice::<Notification>(payload)?)
    }

    #[tracing::instrument(name = "komainu.get_request", skip(self))]
    pub async fn get_request(&self, id: &str) -> Result<Request, KomainuError> {
        self.get_one(&format!("v1/custody/requests/{id}")).await
    }

    #[tracing::instrument(name = "komainu.list_requests", skip(self))]
    pub async fn list_requests(&self) -> Result<Vec<Request>, KomainuError> {
        self.get_many("v1/custody/requests").await
    }

    #[tracing::instrument(name = "komainu.get_transaction", skip(self))]
    pub async fn get_transaction(&self, id: &str) -> Result<Transaction, KomainuError> {
        self.get_one(&format!("v1/custody/transactions/{id}")).await
    }

    #[tracing::instrument(name = "komainu.list_transactions", skip(self))]
    pub async fn list_transactions(&self) -> Result<Vec<Transaction>, KomainuError> {
        self.get_many("v1/custody/transactions").await
    }

    #[tracing::instrument(name = "komainu.get_wallet", skip(self))]
    pub async fn get_wallet(&self, id: &str) -> Result<Wallet, KomainuError> {
        self.get_one(&format!("v1/custody/wallets/{id}")).await
    }

    #[tracing::instrument(name = "komainu.list_wallets", skip(self))]
    pub async fn list_wallets(&self) -> Result<Vec<Wallet>, KomainuError> {
        self.get_many("v1/custody/wallets").await
    }
}

impl KomainuClient {
    async fn get_one<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, KomainuError> {
        self.get(endpoint, None).await
    }

    async fn get_many<T: DeserializeOwned>(&self, endpoint: &str) -> Result<Vec<T>, KomainuError> {
        let mut res = vec![];
        let mut offset = 0;

        loop {
            let page = self.get::<Many<T>>(endpoint, Some(offset)).await?;
            res.extend(page.data);
            if page.has_next {
                offset += 1;
            } else {
                break;
            }
        }

        Ok(res)
    }

    async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        offset: Option<u64>,
    ) -> Result<T, KomainuError> {
        let response = self
            .request::<()>(Method::GET, endpoint, offset, None)
            .await?
            .send()
            .await?
            .json()
            .await?;

        match response {
            Fallible::Error {
                error_code,
                errors,
                status,
            } => Err(KomainuError::KomainuError {
                error_code,
                errors,
                status,
            }),
            Fallible::Ok(res) => Ok(res),
        }
    }

    async fn request<T: Serialize>(
        &self,
        method: Method,
        endpoint: &str,
        offset: Option<u64>,
        payload: Option<T>,
    ) -> Result<RequestBuilder, reqwest::Error> {
        let access_token = self.get_access_token().await?;
        let timestamp = chrono::Utc::now().timestamp_millis();

        let mut url = self.url(endpoint);

        if let Some(offset) = offset {
            url.query_pairs_mut()
                .append_pair("limit", "200")
                .append_pair("offset", &offset.to_string());
        }

        let payload = payload
            .map(|payload| serde_json::to_vec(&payload).expect("encode to JSON"))
            .unwrap_or_default();

        let mut path_query = url.path().to_string();
        if let Some(query) = url.query() {
            path_query.push('?');
            path_query.push_str(query);
        }

        let canonical_string = format!(
            "{},{},{},sha256={},sha256={},{}",
            url.host_str().expect("URL with host"),
            method.as_str().to_lowercase(),
            path_query,
            BASE64_STANDARD.encode(Sha256::digest(&payload)),
            BASE64_STANDARD.encode(Sha256::digest(&access_token)),
            timestamp
        );

        let signature: Signature = self.signing_key.sign(canonical_string.as_bytes());

        Ok(self
            .http_client
            .request(method, url)
            .bearer_auth(access_token)
            .header("X-Timestamp", timestamp)
            .header("X-Signature", BASE64_STANDARD.encode(signature.to_der()))
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(payload))
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

    #[tracing::instrument(name = "komainu.refresh_token", skip(self))]
    async fn refresh_token(&self) -> Result<AccessToken, reqwest::Error> {
        let response: GetTokenResponse = self
            .http_client
            .post(self.url("v1/auth/token"))
            .json(&self.get_token_request)
            .send()
            .await?
            .json()
            .await?;

        Ok(AccessToken {
            access_token: response.access_token,
            expires_at: Instant::now() + Duration::from_secs(response.expires_in),
        })
    }

    fn url(&self, path: &str) -> Url {
        self.endpoint.join(path).expect("valid path")
    }
}

#[derive(Clone)]
struct AccessToken {
    access_token: String,
    expires_at: Instant,
}
