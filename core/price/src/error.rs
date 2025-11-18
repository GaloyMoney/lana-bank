use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("PriceError - BfxClientError: {0}")]
    BfxClientError(#[from] super::bfx_client::error::BfxClientError),
    #[error("PriceError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
    #[error("PriceError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("PriceError - price is not yet available")]
    PriceUnavailable,
}
