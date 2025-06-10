use thiserror::Error;

use crate::state::error::CustodianStateError;

#[derive(Debug, Error)]
pub enum CustodianClientError {
    #[error("CustodianClientError - ClientError: {0}")]
    ClientError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("CustodianClientError - CustodianStateError: {0}")]
    CustodianStateError(#[from] CustodianStateError),
}
