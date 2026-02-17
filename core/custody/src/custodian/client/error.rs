use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::custodian::error::CustodianError;

#[derive(Debug, Error)]
pub enum CustodianClientError {
    #[error("CustodianClientError - ClientError: {0}")]
    ClientError(Box<dyn std::error::Error + Send + Sync>),
}

impl CustodianClientError {
    pub fn client(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl From<bitgo::BitgoError> for CustodianClientError {
    fn from(error: bitgo::BitgoError) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl From<komainu::KomainuError> for CustodianClientError {
    fn from(error: komainu::KomainuError) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl From<money::ConversionError> for CustodianClientError {
    fn from(error: money::ConversionError) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl From<CustodianError> for CustodianClientError {
    fn from(error: CustodianError) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl ErrorSeverity for CustodianClientError {
    fn severity(&self) -> Level {
        match self {
            Self::ClientError(e) => {
                // Try to downcast to known error types that implement ErrorSeverity
                if let Some(bitgo_err) = e.downcast_ref::<bitgo::BitgoError>() {
                    bitgo_err.severity()
                } else if let Some(komainu_err) = e.downcast_ref::<komainu::KomainuError>() {
                    komainu_err.severity()
                } else if let Some(money_err) = e.downcast_ref::<money::ConversionError>() {
                    money_err.severity()
                } else {
                    // Default to ERROR for unknown error types
                    Level::ERROR
                }
            }
        }
    }
}
