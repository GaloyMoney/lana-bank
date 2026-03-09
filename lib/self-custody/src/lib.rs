#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;

pub use config::{SelfCustodyClientConfig, SelfCustodyConfig, SelfCustodyNetwork};
pub use error::SelfCustodyError;

use std::str::FromStr;

use bitcoin::{
    Address, CompressedPublicKey,
    bip32::{ChildNumber, DerivationPath, Xpriv, Xpub},
    key::Secp256k1,
    network::NetworkKind,
};
use money::Satoshis;
use rand::{TryRng as _, rngs::SysRng};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedWallet {
    pub derivation_index: u32,
    pub derivation_path: String,
    pub external_id: String,
    pub address: String,
    pub network: SelfCustodyNetwork,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedAccountKeys {
    pub account_derivation_path: String,
    pub account_xpriv: String,
    pub account_xpub: String,
    pub network: SelfCustodyNetwork,
}

#[derive(Clone)]
pub struct SelfCustodyClient {
    http_client: Client,
    esplora_url: Url,
    account_xpub: Xpub,
    network: SelfCustodyNetwork,
}

impl SelfCustodyClient {
    pub fn try_new(config: SelfCustodyClientConfig) -> Result<Self, SelfCustodyError> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let account_xpub = Xpub::from_str(&config.account_xpub)
            .map_err(|err| SelfCustodyError::InvalidXpub(err.to_string()))?;
        validate_account_xpub_network(account_xpub.network, config.network)?;

        Ok(Self {
            http_client: Client::new(),
            esplora_url: normalize_base_url(config.esplora_url),
            account_xpub,
            network: config.network,
        })
    }

    pub async fn verify(&self) -> Result<(), SelfCustodyError> {
        let tip_height_url = self.esplora_url.join("blocks/tip/height")?;
        let _tip_height: u64 = self
            .http_client
            .get(tip_height_url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?
            .trim()
            .parse()
            .map_err(|_| SelfCustodyError::InvalidEsploraBalance)?;
        Ok(())
    }

    pub fn derive_receive_wallet(
        &self,
        derivation_index: u32,
    ) -> Result<DerivedWallet, SelfCustodyError> {
        let secp = Secp256k1::new();
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_normal_idx(0)?,
            ChildNumber::from_normal_idx(derivation_index)?,
        ]);
        let derived = self.account_xpub.derive_pub(&secp, &derivation_path)?;
        let public_key = CompressedPublicKey::from_slice(&derived.public_key.serialize())
            .map_err(|_| SelfCustodyError::InvalidDerivedPublicKey)?;
        let address = Address::p2wpkh(&public_key, self.network.bitcoin_network());

        Ok(DerivedWallet {
            derivation_index,
            derivation_path: format!(
                "m/{}/0/{derivation_index}",
                self.network.bip84_account_path()
            ),
            external_id: format!("self-custody:{derivation_index}"),
            address: address.to_string(),
            network: self.network,
        })
    }

    pub async fn fetch_confirmed_balance(
        &self,
        address: &str,
    ) -> Result<Satoshis, SelfCustodyError> {
        let address_url = self.esplora_url.join(&format!("address/{address}"))?;
        let response: EsploraAddress = self
            .http_client
            .get(address_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let balance = response
            .chain_stats
            .funded_txo_sum
            .checked_sub(response.chain_stats.spent_txo_sum)
            .ok_or(SelfCustodyError::InvalidEsploraBalance)?;

        Ok(balance.into())
    }
}

impl core::fmt::Debug for SelfCustodyClient {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SelfCustodyClient")
            .field("http_client", &self.http_client)
            .field("esplora_url", &self.esplora_url)
            .field("account_xpub", &"<redacted>")
            .field("network", &self.network)
            .finish()
    }
}

pub fn generate_account_keys(
    network: SelfCustodyNetwork,
) -> Result<GeneratedAccountKeys, SelfCustodyError> {
    let mut seed = [0u8; 64];
    SysRng
        .try_fill_bytes(&mut seed)
        .expect("os rng should be available");

    let secp = Secp256k1::new();
    let account_derivation_path = network.bip84_account_path();
    let account_xpriv = Xpriv::new_master(network.bitcoin_network(), &seed)?
        .derive_priv(&secp, &account_derivation_path)?;
    let account_xpub = Xpub::from_priv(&secp, &account_xpriv);

    Ok(GeneratedAccountKeys {
        account_derivation_path: format!("m/{account_derivation_path}"),
        account_xpriv: account_xpriv.to_string(),
        account_xpub: account_xpub.to_string(),
        network,
    })
}

fn validate_account_xpub_network(
    account_xpub_network: NetworkKind,
    selected_network: SelfCustodyNetwork,
) -> Result<(), SelfCustodyError> {
    // Extended key version bytes only distinguish mainnet from the test-family networks.
    if account_xpub_network != selected_network.xpub_network_kind() {
        return Err(SelfCustodyError::XpubNetworkMismatch {
            expected: selected_network,
            actual: account_xpub_network,
        });
    }

    Ok(())
}

fn normalize_base_url(mut url: Url) -> Url {
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path().trim_end_matches('/'));
        url.set_path(&path);
    }
    url
}

#[derive(Deserialize)]
struct EsploraAddress {
    chain_stats: EsploraStats,
}

#[derive(Deserialize)]
struct EsploraStats {
    funded_txo_sum: u64,
    spent_txo_sum: u64,
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };

    use axum::{
        Json, Router,
        extract::{Path, State},
        response::IntoResponse,
        routing::get,
    };

    use super::*;

    #[test]
    fn self_custody_config_debug_redacts_account_xpub() {
        let generated =
            generate_account_keys(SelfCustodyNetwork::Mainnet).expect("key generation succeeds");
        let debug = format!(
            "{:?}",
            SelfCustodyConfig {
                account_xpub: generated.account_xpub.clone(),
                network: SelfCustodyNetwork::Mainnet,
            }
        );

        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains(&generated.account_xpub));
    }

    #[test]
    fn generate_account_keys_produces_valid_account_xpub() {
        let generated =
            generate_account_keys(SelfCustodyNetwork::Testnet4).expect("key generation succeeds");
        let client = SelfCustodyClient::try_new(SelfCustodyClientConfig {
            account_xpub: generated.account_xpub.clone(),
            network: SelfCustodyNetwork::Testnet4,
            esplora_url: Url::parse("http://127.0.0.1:3001").expect("valid url"),
        })
        .expect("generated xpub is valid");

        let wallet = client
            .derive_receive_wallet(0)
            .expect("first receive address derives");
        assert!(wallet.address.starts_with("tb1"));
        assert_eq!(wallet.derivation_path, "m/84'/1'/0'/0/0");
    }

    #[test]
    fn derive_receive_wallet_uses_unique_addresses_per_index() {
        let generated =
            generate_account_keys(SelfCustodyNetwork::Mainnet).expect("key generation succeeds");
        let client = SelfCustodyClient::try_new(SelfCustodyClientConfig {
            account_xpub: generated.account_xpub,
            network: SelfCustodyNetwork::Mainnet,
            esplora_url: Url::parse("http://127.0.0.1:3001").expect("valid url"),
        })
        .expect("generated xpub is valid");

        let first = client
            .derive_receive_wallet(0)
            .expect("first address derives");
        let second = client
            .derive_receive_wallet(1)
            .expect("second address derives");

        assert_ne!(first.address, second.address);
        assert_eq!(first.derivation_path, "m/84'/0'/0'/0/0");
        assert_eq!(second.derivation_path, "m/84'/0'/0'/0/1");
    }

    #[test]
    fn try_new_rejects_xpub_from_another_network_family() {
        let generated =
            generate_account_keys(SelfCustodyNetwork::Mainnet).expect("key generation succeeds");
        let error = SelfCustodyClient::try_new(SelfCustodyClientConfig {
            account_xpub: generated.account_xpub,
            network: SelfCustodyNetwork::Signet,
            esplora_url: Url::parse("http://127.0.0.1:3001").expect("valid url"),
        })
        .expect_err("mainnet xpub must be rejected for signet");

        assert!(matches!(
            error,
            SelfCustodyError::XpubNetworkMismatch {
                expected: SelfCustodyNetwork::Signet,
                actual: NetworkKind::Main,
            }
        ));
    }

    #[test]
    fn generate_account_keys_and_addresses_support_signet() {
        let generated =
            generate_account_keys(SelfCustodyNetwork::Signet).expect("key generation succeeds");
        let client = SelfCustodyClient::try_new(SelfCustodyClientConfig {
            account_xpub: generated.account_xpub,
            network: SelfCustodyNetwork::Signet,
            esplora_url: Url::parse("http://127.0.0.1:3001").expect("valid url"),
        })
        .expect("generated xpub is valid");

        let wallet = client
            .derive_receive_wallet(0)
            .expect("first receive address derives");

        assert!(wallet.address.starts_with("tb1"));
        assert_eq!(wallet.derivation_path, "m/84'/1'/0'/0/0");
        assert_eq!(wallet.network, SelfCustodyNetwork::Signet);
    }

    #[tokio::test]
    async fn verify_and_fetch_confirmed_balance_use_esplora_endpoints() {
        let generated =
            generate_account_keys(SelfCustodyNetwork::Testnet3).expect("key generation succeeds");
        let state = TestState {
            balances: Arc::new(AtomicU64::new(250_000)),
        };
        let server = TestServer::spawn(state.clone()).await;
        let client = SelfCustodyClient::try_new(SelfCustodyClientConfig {
            account_xpub: generated.account_xpub,
            network: SelfCustodyNetwork::Testnet3,
            esplora_url: server.base_url.clone(),
        })
        .expect("generated xpub is valid");

        client.verify().await.expect("tip height endpoint responds");

        let wallet = client
            .derive_receive_wallet(3)
            .expect("receive address derives");
        let balance = client
            .fetch_confirmed_balance(&wallet.address)
            .await
            .expect("balance fetch succeeds");

        assert_eq!(balance, Satoshis::from(250_000));
        server.shutdown().await;
    }

    #[derive(Clone)]
    struct TestState {
        balances: Arc<AtomicU64>,
    }

    struct TestServer {
        base_url: Url,
        shutdown: tokio::sync::oneshot::Sender<()>,
        handle: tokio::task::JoinHandle<()>,
    }

    impl TestServer {
        async fn spawn(state: TestState) -> Self {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("listener binds");
            let addr: SocketAddr = listener.local_addr().expect("listener has local addr");
            let base_url = Url::parse(&format!("http://{addr}")).expect("base url parses");

            let app = Router::new()
                .route("/blocks/tip/height", get(|| async { "42" }))
                .route("/address/{address}", get(address))
                .with_state(state);

            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
            let handle = tokio::spawn(async move {
                let _ = axum::serve(listener, app)
                    .with_graceful_shutdown(async {
                        let _ = shutdown_rx.await;
                    })
                    .await;
            });

            Self {
                base_url,
                shutdown: shutdown_tx,
                handle,
            }
        }

        async fn shutdown(self) {
            let _ = self.shutdown.send(());
            let _ = self.handle.await;
        }
    }

    async fn address(
        State(state): State<TestState>,
        Path(_address): Path<String>,
    ) -> impl IntoResponse {
        Json(serde_json::json!({
            "chain_stats": {
                "funded_txo_sum": state.balances.load(Ordering::SeqCst),
                "spent_txo_sum": 0u64
            }
        }))
    }
}
