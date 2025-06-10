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
impl CustodianClient for komainu::KomainuClient {
    async fn create_address(&self, label: &str) -> Result<AddressResponse, CustodianClientError> {
        todo!()
    }
}
