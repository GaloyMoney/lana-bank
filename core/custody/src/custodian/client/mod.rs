pub mod error;

use async_trait::async_trait;
use serde_json::Value;

use error::CustodianClientError;

pub struct WalletResponse {
    pub external_id: String,
    pub address: String,
    pub full_response: serde_json::Value,
}

#[async_trait]
pub trait CustodianClient: Send {
    async fn initialize_wallet(&self, label: &str) -> Result<WalletResponse, CustodianClientError>;

    async fn process_webhook(&self, payload: Value) -> Result<(), CustodianClientError>;
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

    async fn process_webhook(&self, _payload: Value) -> Result<(), CustodianClientError> {
        Ok(())
    }
}

#[async_trait]
impl CustodianClient for komainu::KomainuClient {
    async fn initialize_wallet(
        &self,
        _label: &str,
    ) -> Result<WalletResponse, CustodianClientError> {
        todo!()
    }

    async fn process_webhook(&self, _payload: Value) -> Result<(), CustodianClientError> {
        Ok(())
    }
}

#[cfg(feature = "mock-custodian")]
pub mod mock {
    use async_trait::async_trait;

    use super::*;

    pub struct CustodianMock;

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

        async fn process_webhook(&self, _payload: Value) -> Result<(), CustodianClientError> {
            Ok(())
        }
    }
}
