use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use super::repo::{
    PriceProviderCreateError, PriceProviderFindError, PriceProviderModifyError,
    PriceProviderQueryError,
};

#[derive(Error, Debug)]
pub enum PriceProviderError {
    #[error("PriceProviderError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PriceProviderError - Create: {0}")]
    Create(#[from] PriceProviderCreateError),
    #[error("PriceProviderError - Modify: {0}")]
    Modify(#[from] PriceProviderModifyError),
    #[error("PriceProviderError - Find: {0}")]
    Find(#[from] PriceProviderFindError),
    #[error("PriceProviderError - Query: {0}")]
    Query(#[from] PriceProviderQueryError),
    #[error("PriceProviderError - BfxClientError: {0}")]
    BfxClientError(#[from] bfx_client::BfxClientError),
    #[error("PriceProviderError - ConversionError: {0}")]
    ConversionError(#[from] money::ConversionError),
    #[error("PriceProviderError - NoActiveProviders")]
    NoActiveProviders,
    #[error("PriceProviderError - AllProvidersFailed")]
    AllProvidersFailed,
}

impl ErrorSeverity for PriceProviderError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::BfxClientError(e) => e.severity(),
            Self::ConversionError(e) => e.severity(),
            Self::NoActiveProviders => Level::WARN,
            Self::AllProvidersFailed => Level::ERROR,
        }
    }
}
