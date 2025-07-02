pub mod error;

use async_trait::async_trait;

use error::CustodianClientError;

pub struct AddressResponse {
    pub address: String,
    pub full_response: serde_json::Value,
}

pub struct WalletResponse {
    pub external_id: String,
    pub full_response: serde_json::Value,
}

#[async_trait]
pub trait CustodianClient: Send {
    async fn initialize_wallet<'a>(
        &self,
        label: &str,
    ) -> Result<WalletResponse, CustodianClientError>;

    async fn get_address<'a>(
        &self,
        external_wallet_id: &str,
    ) -> Result<AddressResponse, CustodianClientError>;
}

#[async_trait]
impl CustodianClient for bitgo::BitgoClient {
    async fn initialize_wallet<'a>(
        &self,
        label: &str,
    ) -> Result<WalletResponse, CustodianClientError> {
        let (wallet, full_response) = self
            .generate_wallet(label)
            .await
            .map_err(CustodianClientError::client)?;

        Ok(WalletResponse {
            external_id: wallet.id,
            full_response,
        })
    }

    async fn get_address<'a>(
        &self,
        external_wallet_id: &str,
    ) -> Result<AddressResponse, CustodianClientError> {
        let (wallet, full_response) = self
            .get_wallet(external_wallet_id)
            .await
            .map_err(CustodianClientError::client)?;

        Ok(AddressResponse {
            address: wallet.receive_address.address,
            full_response,
        })
    }
}

#[async_trait]
impl CustodianClient for komainu::KomainuClient {
    async fn initialize_wallet<'a>(
        &self,
        _label: &str,
    ) -> Result<WalletResponse, CustodianClientError> {
        todo!()
    }

    async fn get_address<'a>(
        &self,
        _external_wallet_id: &str,
    ) -> Result<AddressResponse, CustodianClientError> {
        todo!()
    }
}

#[cfg(feature = "mock-custodian")]
pub mod mock {
    use async_trait::async_trait;

    use super::*;

    pub struct CustodianMock;

    #[async_trait]
    impl CustodianClient for CustodianMock {
        async fn create_address<'a>(
            &self,
            label: &str,
            _state: CustodianStateRepo<'a>,
        ) -> Result<AddressResponse, CustodianClientError> {
            Ok(AddressResponse {
                address: "bt1qaddressmock".to_string(),
                label: label.to_string(),
                full_response: serde_json::Value::Null,
            })
        }
    }
}
