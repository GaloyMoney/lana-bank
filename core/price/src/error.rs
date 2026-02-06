use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("PriceError - BfxClientError: {0}")]
    BfxClientError(#[from] bfx_client::BfxClientError),
    #[error("PriceError - ConversionError: {0}")]
    ConversionError(#[from] money::ConversionError),
    #[error("PriceError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("PriceError - Price not yet available")]
    PriceUnavailable,
}

impl ErrorSeverity for PriceError {
    fn severity(&self) -> Level {
        match self {
            Self::BfxClientError(e) => e.severity(),
            Self::ConversionError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
            Self::PriceUnavailable => Level::WARN,
        }
    }
}
