use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("PriceError - PriceClientError: {0}")]
    PriceClientError(#[from] crate::price_client::error::PriceClientError),
    #[error("PriceError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
}
