pub mod error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use error::CustodianClientError;

use super::state::repo::PersistedCustodianState;

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
        state: PersistedCustodianState<'a>,
    ) -> Result<AddressResponse, CustodianClientError>;
}

#[async_trait]
impl CustodianClient for komainu::KomainuClient {
    async fn create_address<'a>(
        &self,
        label: &str,
        state: PersistedCustodianState<'a>,
    ) -> Result<AddressResponse, CustodianClientError> {
        let mut komainu_state: KomainuState = state.load().await?;

        todo!("call komainu");

        komainu_state.latest_used_address_index += 1;
        state.persist(&komainu_state).await?;

        todo!("return address")
    }
}

#[derive(Serialize, Deserialize, Default)]
struct KomainuState {
    latest_used_address_index: u64,
}
