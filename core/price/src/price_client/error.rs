use thiserror::Error;

#[derive(Debug, Error)]
pub enum PriceClientError {
    #[error("PriceClientError - ClientError: {0}")]
    ClientError(Box<dyn std::error::Error + Send + Sync>),
}

impl PriceClientError {
    pub fn client(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl From<bitfinex::BfxClientError> for PriceClientError {
    fn from(error: bitfinex::BfxClientError) -> Self {
        Self::ClientError(Box::new(error))
    }
}
