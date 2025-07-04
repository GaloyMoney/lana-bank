use thiserror::Error;

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
