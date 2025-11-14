use thiserror::Error;

use crate::sources::error::PriceClientError;

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("PriceError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
    #[error("PriceError - ClientError: {0}")]
    ClientError(#[from] PriceClientError),
    #[error("PriceError - MissingPrice")]
    MissingPrice,
    #[error("PriceError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PriceError - NoPriceSourcesAvailable")]
    NoPriceSourcesAvailable,
}
