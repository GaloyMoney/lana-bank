use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("PriceError - BfxClientError: {0}")]
    BfxClientError(#[from] super::bfx_client::error::BfxClientError),
    #[error("PriceError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
}
