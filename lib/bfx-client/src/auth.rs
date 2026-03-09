use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use reqwest::Client as ReqwestClient;
use sha2::Sha384;
use tracing_macros::record_error_severity;

use crate::config::{BfxAuthConfig, BfxDirectoryConfig};
use crate::error::BfxClientError;
use crate::response::{BfxNotification, DepositAddress, Wallet};

type HmacSha384 = Hmac<Sha384>;

pub struct BfxAuthClient {
    client: ReqwestClient,
    api_key: String,
    api_secret: String,
    api_url: url::Url,
    last_nonce: AtomicU64,
}

impl BfxAuthClient {
    pub fn try_new(
        config: BfxAuthConfig,
        directory: BfxDirectoryConfig,
    ) -> Result<Self, BfxClientError> {
        let _ = rustls::crypto::ring::default_provider().install_default();

        Ok(Self {
            client: ReqwestClient::builder().use_rustls_tls().build()?,
            api_key: config.api_key,
            api_secret: config.api_secret,
            api_url: directory.api_url,
            last_nonce: AtomicU64::new(0),
        })
    }

    #[record_error_severity]
    #[tracing::instrument(name = "bfx.auth.get_wallets", skip(self), fields(url, response))]
    pub async fn get_wallets(&self) -> Result<Vec<Wallet>, BfxClientError> {
        self.post("auth/r/wallets", None).await
    }

    #[record_error_severity]
    #[tracing::instrument(
        name = "bfx.auth.get_deposit_address",
        skip(self),
        fields(url, response, wallet, method, renew)
    )]
    pub async fn get_deposit_address(
        &self,
        wallet: &str,
        method: &str,
        op_renew: bool,
    ) -> Result<BfxNotification<DepositAddress>, BfxClientError> {
        let body = serde_json::json!({
            "wallet": wallet,
            "method": method,
            "op_renew": u8::from(op_renew),
        });

        self.post("auth/w/deposit/address", Some(body)).await
    }

    async fn post<T>(
        &self,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T, BfxClientError>
    where
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let endpoint = endpoint.trim_start_matches('/');
        let body = body
            .map(|value| serde_json::to_string(&value))
            .transpose()?;
        let url = format!("{}/{endpoint}", self.api_url.as_str().trim_end_matches('/'));
        tracing::Span::current().record("url", tracing::field::display(&url));
        let headers = self.auth_headers(endpoint, body.as_deref())?;

        let mut request = self
            .client
            .post(&url)
            .headers(headers)
            .header("accept", "application/json")
            .header("content-type", "application/json");

        if let Some(body) = body {
            request = request.body(body);
        }

        let response = request.send().await?;
        let data = crate::extract_response_data::<T>(response).await?;
        tracing::Span::current().record("response", tracing::field::debug(&data));

        Ok(data)
    }

    fn auth_headers(
        &self,
        endpoint: &str,
        body: Option<&str>,
    ) -> Result<reqwest::header::HeaderMap, BfxClientError> {
        let nonce = self.next_nonce();
        let payload = match body {
            Some(body) => format!("/api/v2/{endpoint}{nonce}{body}"),
            None => format!("/api/v2/{endpoint}{nonce}"),
        };

        let mut signature = HmacSha384::new_from_slice(self.api_secret.as_bytes())
            .expect("SHA384 accepts arbitrary key sizes");
        signature.update(payload.as_bytes());

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "bfx-nonce",
            nonce
                .to_string()
                .parse()
                .expect("valid Bitfinex nonce header"),
        );
        headers.insert(
            "bfx-signature",
            hex::encode(signature.finalize().into_bytes())
                .parse()
                .expect("valid Bitfinex signature header"),
        );
        headers.insert(
            "bfx-apikey",
            self.api_key.parse().expect("valid Bitfinex api key header"),
        );

        Ok(headers)
    }

    fn next_nonce(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_micros() as u64;

        next_monotonic_nonce(&self.last_nonce, now)
    }
}

fn next_monotonic_nonce(last_nonce: &AtomicU64, now: u64) -> u64 {
    let mut current = last_nonce.load(Ordering::Relaxed);

    loop {
        let next = now.max(current.saturating_add(1));
        match last_nonce.compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => return next,
            Err(observed) => current = observed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_matches_bitfinex_auth_format() {
        let endpoint = "auth/r/wallets";
        let nonce = 1700000000000000_u64;
        let mut signature = HmacSha384::new_from_slice(b"top-secret").unwrap();
        signature.update(format!("/api/v2/{endpoint}{nonce}").as_bytes());

        assert_eq!(
            hex::encode(signature.finalize().into_bytes()),
            "0fa8c15e7052c481951fe9b4d2297a0744d435352c61d6e62a8a31304def66327cbb4739fb4f7caada08e500d6facd44"
        );
    }

    #[test]
    fn nonce_is_monotonic_when_clock_stalls() {
        let last_nonce = AtomicU64::new(42);

        assert_eq!(next_monotonic_nonce(&last_nonce, 40), 43);
        assert_eq!(next_monotonic_nonce(&last_nonce, 43), 44);
        assert_eq!(next_monotonic_nonce(&last_nonce, 100), 100);
    }
}
