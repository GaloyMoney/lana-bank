pub mod error;

use async_trait::async_trait;

use error::CustodianClientError;

pub struct AddressResponse {
    pub address: String,
    pub label: String,
    pub full_response: serde_json::Value,
}

#[async_trait]
pub trait CustodianClient: Send {
    async fn create_address(&self, label: &str) -> Result<AddressResponse, CustodianClientError>;
}

#[async_trait]
impl CustodianClient for bitgo::BitgoClient {
    async fn create_address(&self, label: &str) -> Result<AddressResponse, CustodianClientError> {
        let (address, response) = self
            .create_address()
            .await
            .map_err(|e| CustodianClientError::ClientError(Box::new(e)))?;
        Ok(AddressResponse {
            address: address.address,
            label: label.to_owned(),
            full_response: response,
        })
    }
}

// #[async_trait]
// impl CustodianClient for komainu::KomainuClient {
//     async fn create_address(&self, label: &str) -> Result<AddressResponse, CustodianClientError> {
//     }
// }
