use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("LoanError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PriceError - BfxClientError: {0}")]
    BfxClientError(#[from] super::bfx_client::error::BfxClientError),
    #[error("PriceError - ConversionError: {0}")]
    ConversionError(#[from] crate::primitives::ConversionError),
    #[error("LoanError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
}
