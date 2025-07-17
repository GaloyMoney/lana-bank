pub mod error;

use async_trait::async_trait;
use bitgo::TransferState;
use bytes::Bytes;

use core_money::Satoshis;

use error::CustodianClientError;

use super::notification::CustodianNotification;

pub struct WalletResponse {
    pub external_id: String,
    pub address: String,
    pub full_response: serde_json::Value,
}

#[async_trait]
pub trait CustodianClient: Send {
    async fn initialize_wallet(&self, label: &str) -> Result<WalletResponse, CustodianClientError>;

    async fn process_webhook(
        &self,
        headers: &http::HeaderMap,
        payload: Bytes,
    ) -> Result<Option<CustodianNotification>, CustodianClientError>;
}

#[async_trait]
impl CustodianClient for bitgo::BitgoClient {
    async fn initialize_wallet(&self, label: &str) -> Result<WalletResponse, CustodianClientError> {
        let (wallet, full_response) = self
            .generate_wallet(label)
            .await
            .map_err(CustodianClientError::client)?;

        Ok(WalletResponse {
            external_id: wallet.id,
            address: wallet.receive_address.address,
            full_response,
        })
    }

    async fn process_webhook(
        &self,
        headers: &http::HeaderMap,
        payload: Bytes,
    ) -> Result<Option<CustodianNotification>, CustodianClientError> {
        let notification = self
            .validate_webhook_notification(headers, &payload)
            .map_err(CustodianClientError::client)?;

        use bitgo::Notification;

        let custodian_notification = match notification {
            Notification::Transfer(transfer) if transfer.state == TransferState::Confirmed => {
                let transfer = self
                    .get_transfer(&transfer.wallet, &transfer.transfer)
                    .await
                    .map_err(CustodianClientError::client)?;

                if transfer.state == TransferState::Confirmed {
                    let (wallet, _) = self
                        .get_wallet(&transfer.wallet)
                        .await
                        .map_err(CustodianClientError::client)?;

                    Some(CustodianNotification::WalletBalanceChanged {
                        external_wallet_id: transfer.wallet,
                        amount: wallet.confirmed_balance.into(),
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
    async fn initialize_wallet(
        &self,
        _label: &str,
    ) -> Result<WalletResponse, CustodianClientError> {
        Ok(WalletResponse {
            external_id: "c33f1d21-6e42-49e6-a676-17ad0ac6dfef".to_string(),
            address: "tb1qplx6wllreywl3nadc7wh6waah58xq7p48857qh".to_string(),
            full_response: serde_json::Value::Null,
        })
    }

    async fn process_webhook(
        &self,
        headers: &http::HeaderMap,
        payload: Bytes,
    ) -> Result<Option<CustodianNotification>, CustodianClientError> {
        let notification = self
            .validate_webhook_notification(headers, &payload)
            .map_err(CustodianClientError::client)?;

        use komainu::{EntityType, EventType, Notification};

        let custodian_notification = match notification {
            Notification {
                event_type: EventType::BalanceUpdated,
                entity: EntityType::Wallet,
                entity_id: wallet_id,
            } => {
                let wallet = self
                    .get_wallet(&wallet_id)
                    .await
                    .map_err(CustodianClientError::client)?;

                let amount = Satoshis::try_from_btc(wallet.balance.available)
                    .map_err(CustodianClientError::client)?;

                Some(CustodianNotification::WalletBalanceChanged {
                    external_wallet_id: wallet.id,
                    amount,
                })
            }
        };

        Ok(custodian_notification)
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
        async fn initialize_wallet(
            &self,
            _label: &str,
        ) -> Result<WalletResponse, CustodianClientError> {
            Ok(WalletResponse {
                external_id: "123".to_string(),
                address: "bt1qaddressmock".to_string(),
                full_response: serde_json::Value::Null,
            })
        }
        async fn process_webhook(
            &self,
            _headers: &http::HeaderMap,
            payload: Bytes,
        ) -> Result<Option<CustodianNotification>, CustodianClientError> {
            if let Ok(WalletBalanceChanged { wallet, balance }) = serde_json::from_slice(&payload) {
                Ok(Some(CustodianNotification::WalletBalanceChanged {
                    external_wallet_id: wallet,
                    amount: balance.into(),
                }))
            } else {
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}
