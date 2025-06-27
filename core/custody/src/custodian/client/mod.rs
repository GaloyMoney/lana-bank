pub mod error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use error::CustodianClientError;

use super::state::repo::CustodianStateRepo;

pub struct AddressResponse {
    pub address: String,
    pub label: String,
    pub full_response: serde_json::Value,
}

#[async_trait]
pub trait CustodianClient: Send {
    async fn create_address<'a>(
        &self,
        label: &str,
        state: CustodianStateRepo<'a>,
    ) -> Result<AddressResponse, CustodianClientError>;
}

#[async_trait]
impl CustodianClient for komainu::KomainuClient {
    async fn create_address<'a>(
        &self,
        label: &str,
        state: CustodianStateRepo<'a>,
    ) -> Result<AddressResponse, CustodianClientError> {
        let mut komainu_state: KomainuState = state.load().await?;

        // let address = self.get_wallet_address_by_index(komainu_state.index).await?;

        komainu_state.first_unused_address_index += 1;
        state.persist(&komainu_state).await?;

        Ok(AddressResponse {
            address: "bt1qaddress".to_string(),
            label: label.to_string(),
            full_response: serde_json::Value::Null,
        })
    }
}

#[derive(Serialize, Deserialize, Default)]
struct KomainuState {
    first_unused_address_index: u64,
}
