use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceClientError {
    #[error("PriceClientError - BfxClientError: {0}")]
    BfxClientError(#[from] crate::bfx_client::error::BfxClientError),
}
