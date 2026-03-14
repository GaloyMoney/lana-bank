pub mod error;

use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;

use old_money::Satoshis;

use error::CustodianClientError;

use crate::primitives::{ExternalWallet, WalletNetwork};

use super::notification::CustodianNotification;

#[async_trait]
pub trait CustodianClient: Send {
    /// Performs an authenticated call to the custodian to verify
    /// correctness of the configuration.
    async fn verify_client(&self) -> Result<(), CustodianClientError>;

    /// Performs initialization of a wallet on the custodian.
    /// This call may or may not create new wallet.
    /// Returns None if no wallet was created (e.g., Manual custodian).
    async fn initialize_wallet(
        &self,
        label: &str,
        receive_index: Option<u32>,
    ) -> Result<Option<ExternalWallet>, CustodianClientError>;

    /// Fetches the confirmed balance for a wallet when the provider supports polling.
    async fn fetch_wallet_balance(
        &self,
        external_wallet_id: &str,
        address: &str,
    ) -> Result<Option<Satoshis>, CustodianClientError>;

    /// Validates and parses webhook.
    async fn process_webhook(
        &self,
        headers: &http::HeaderMap,
        payload: Bytes,
    ) -> Result<Option<CustodianNotification>, CustodianClientError>;
}

#[async_trait]
impl CustodianClient for bitgo::BitgoClient {
    async fn verify_client(&self) -> Result<(), CustodianClientError> {
        let _ = self.get_wallet_count().await?;
        let _ = self.get_enterprise().await?;
        Ok(())
    }

    async fn initialize_wallet(
        &self,
        label: &str,
        _receive_index: Option<u32>,
    ) -> Result<Option<ExternalWallet>, CustodianClientError> {
        let (wallet, full_response) = self.add_wallet(label).await?;
        let network = if self.is_testnet() {
            WalletNetwork::Testnet4
        } else {
            WalletNetwork::Mainnet
        };

        Ok(Some(ExternalWallet {
            external_id: wallet.id,
            address: wallet.receive_address.address,
            network,
            full_response,
        }))
    }

    async fn fetch_wallet_balance(
        &self,
        _external_wallet_id: &str,
        _address: &str,
    ) -> Result<Option<Satoshis>, CustodianClientError> {
        Ok(None)
    }

    async fn process_webhook(
        &self,
        headers: &http::HeaderMap,
        payload: Bytes,
    ) -> Result<Option<CustodianNotification>, CustodianClientError> {
        let notification = self.validate_webhook_notification(headers, &payload)?;

        use bitgo::{Notification, TransferState};

        let custodian_notification = match notification {
            Notification::Transfer(transfer) if transfer.simulation => None,
            Notification::Transfer(transfer) if transfer.state == TransferState::Confirmed => {
                let transfer = self
                    .get_transfer(&transfer.transfer, &transfer.wallet)
                    .await?;

                if transfer.state == TransferState::Confirmed {
                    let (wallet, _) = self.get_wallet(&transfer.wallet).await?;

                    let changed_at = transfer.confirmed_time.unwrap_or_else(Utc::now);

                    Some(CustodianNotification::WalletBalanceChanged {
                        external_wallet_id: transfer.wallet,
                        new_balance: wallet.confirmed_balance.into(),
                        changed_at,
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        Ok(custodian_notification)
    }
}

#[async_trait]
impl CustodianClient for komainu::KomainuClient {
    async fn verify_client(&self) -> Result<(), CustodianClientError> {
        let _ = self.list_wallets().await?;
        Ok(())
    }

    async fn initialize_wallet(
        &self,
        _label: &str,
        _receive_index: Option<u32>,
    ) -> Result<Option<ExternalWallet>, CustodianClientError> {
        Ok(Some(ExternalWallet {
            external_id: "efabc792-a0fe-44b6-b0b5-4966997e8962".to_string(),
            address: "tb1qplx6wllreywl3nadc7wh6waah58xq7p48857qh".to_string(),
            network: WalletNetwork::Testnet3,
            full_response: serde_json::Value::Null,
        }))
    }

    async fn fetch_wallet_balance(
        &self,
        _external_wallet_id: &str,
        _address: &str,
    ) -> Result<Option<Satoshis>, CustodianClientError> {
        Ok(None)
    }

    async fn process_webhook(
        &self,
        headers: &http::HeaderMap,
        payload: Bytes,
    ) -> Result<Option<CustodianNotification>, CustodianClientError> {
        let notification = self.validate_webhook_notification(headers, &payload)?;

        use komainu::{EntityType, EventType, Notification};

        let custodian_notification = match notification {
            Notification {
                event_type: EventType::BalanceUpdated,
                entity: EntityType::Wallet,
                entity_id: wallet_id,
            } => {
                let wallet = self.get_wallet(&wallet_id).await?;

                let new_balance = Satoshis::try_from_btc(wallet.balance.available)?;

                let changed_at = wallet.balance.balance_updated_at.unwrap_or_else(Utc::now);

                Some(CustodianNotification::WalletBalanceChanged {
                    external_wallet_id: wallet.id,
                    new_balance,
                    changed_at,
                })
            }
        };

        Ok(custodian_notification)
    }
}

#[async_trait]
impl CustodianClient for self_custody::SelfCustodyClient {
    async fn verify_client(&self) -> Result<(), CustodianClientError> {
        self.verify().await?;
        Ok(())
    }

    async fn initialize_wallet(
        &self,
        _label: &str,
        receive_index: Option<u32>,
    ) -> Result<Option<ExternalWallet>, CustodianClientError> {
        let receive_index = receive_index.ok_or_else(|| {
            CustodianClientError::client(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "self-custody wallet creation requires a receive index",
            ))
        })?;

        let wallet = self.derive_receive_wallet(receive_index)?;

        Ok(Some(ExternalWallet {
            external_id: wallet.external_id,
            address: wallet.address,
            network: match wallet.network {
                self_custody::SelfCustodyNetwork::Testnet3 => WalletNetwork::Testnet3,
                self_custody::SelfCustodyNetwork::Testnet4 => WalletNetwork::Testnet4,
                self_custody::SelfCustodyNetwork::Signet => WalletNetwork::Signet,
                self_custody::SelfCustodyNetwork::Mainnet => WalletNetwork::Mainnet,
            },
            full_response: serde_json::json!({
                "derivation_index": wallet.derivation_index,
                "derivation_path": wallet.derivation_path,
                "type": "self_custody"
            }),
        }))
    }

    async fn fetch_wallet_balance(
        &self,
        _external_wallet_id: &str,
        address: &str,
    ) -> Result<Option<Satoshis>, CustodianClientError> {
        Ok(Some(self.fetch_confirmed_balance(address).await?))
    }

    async fn process_webhook(
        &self,
        _headers: &http::HeaderMap,
        _payload: Bytes,
    ) -> Result<Option<CustodianNotification>, CustodianClientError> {
        Ok(None)
    }
}

#[cfg(feature = "mock-custodian")]
pub mod mock {
    use async_trait::async_trait;
    use serde::Deserialize;

    use super::*;

    pub struct CustodianMock;

    #[derive(Deserialize)]
    struct WalletBalanceChanged {
        wallet: String,
        balance: u64,
    }

    #[async_trait]
    impl CustodianClient for CustodianMock {
        async fn verify_client(&self) -> Result<(), CustodianClientError> {
            Ok(())
        }

        async fn initialize_wallet(
            &self,
            _label: &str,
            _receive_index: Option<u32>,
        ) -> Result<Option<ExternalWallet>, CustodianClientError> {
            Ok(Some(ExternalWallet {
                external_id: "123".to_string(),
                address: "bt1qaddressmock".to_string(),
                network: WalletNetwork::Testnet4,
                full_response: serde_json::Value::Null,
            }))
        }

        async fn fetch_wallet_balance(
            &self,
            _external_wallet_id: &str,
            _address: &str,
        ) -> Result<Option<Satoshis>, CustodianClientError> {
            Ok(None)
        }

        async fn process_webhook(
            &self,
            _headers: &http::HeaderMap,
            payload: Bytes,
        ) -> Result<Option<CustodianNotification>, CustodianClientError> {
            if let Ok(WalletBalanceChanged { wallet, balance }) = serde_json::from_slice(&payload) {
                Ok(Some(CustodianNotification::WalletBalanceChanged {
                    external_wallet_id: wallet,
                    new_balance: balance.into(),
                    changed_at: Utc::now(),
                }))
            } else {
                Ok(None)
            }
        }
    }
}

pub struct ManualCustodian;

#[async_trait]
impl CustodianClient for ManualCustodian {
    async fn verify_client(&self) -> Result<(), CustodianClientError> {
        Ok(())
    }

    async fn fetch_wallet_balance(
        &self,
        _external_wallet_id: &str,
        _address: &str,
    ) -> Result<Option<Satoshis>, CustodianClientError> {
        Ok(None)
    }

    async fn initialize_wallet(
        &self,
        _label: &str,
        _receive_index: Option<u32>,
    ) -> Result<Option<ExternalWallet>, CustodianClientError> {
        Ok(None)
    }

    async fn process_webhook(
        &self,
        _headers: &http::HeaderMap,
        _payload: Bytes,
    ) -> Result<Option<CustodianNotification>, CustodianClientError> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    #[test]
    fn hmac_sha256_signature() {
        // https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries#testing-the-webhook-payload-validation

        let mut mac = Hmac::<Sha256>::new_from_slice(b"It's a Secret to Everybody").unwrap();

        mac.update(b"Hello, World!");

        assert_eq!(
            mac.verify_slice(&[
                0x75, 0x71, 0x07, 0xea, 0x0e, 0xb2, 0x50, 0x9f, 0xc2, 0x11, 0x22, 0x1c, 0xce, 0x98,
                0x4b, 0x8a, 0x37, 0x57, 0x0b, 0x6d, 0x75, 0x86, 0xc2, 0x2c, 0x46, 0xf4, 0x37, 0x9c,
                0x8b, 0x04, 0x3e, 0x17,
            ]),
            Ok(())
        );
    }

    #[tokio::test]
    async fn self_custody_signet_wallets_map_to_signet_network() {
        let generated =
            self_custody::generate_account_keys(self_custody::SelfCustodyNetwork::Signet)
                .expect("key generation succeeds");
        let client =
            self_custody::SelfCustodyClient::try_new(self_custody::SelfCustodyClientConfig {
                account_xpub: generated.account_xpub,
                network: self_custody::SelfCustodyNetwork::Signet,
                esplora_url: Url::parse("http://127.0.0.1:3001").expect("valid url"),
            })
            .expect("generated xpub is valid");

        let wallet = client
            .initialize_wallet("signet-loan", Some(7))
            .await
            .expect("wallet initializes");

        assert_eq!(wallet.unwrap().network, WalletNetwork::Signet);
    }
}
