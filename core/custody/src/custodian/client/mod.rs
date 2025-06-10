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
    async fn create_address(
        &self,
        label: &str,
        state: serde_json::Value,
    ) -> Result<(AddressResponse, serde_json::Value), CustodianClientError>;
}

#[async_trait]
impl CustodianClient for komainu::KomainuClient {
    async fn create_address(
        &self,
        label: &str,
        state: serde_json::Value,
    ) -> Result<(AddressResponse, serde_json::Value), CustodianClientError> {
        todo!()
    }
}
