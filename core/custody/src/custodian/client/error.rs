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

impl From<bitgo::BitgoError> for CustodianClientError {
    fn from(error: bitgo::BitgoError) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl From<komainu::KomainuError> for CustodianClientError {
    fn from(error: komainu::KomainuError) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl From<core_money::ConversionError> for CustodianClientError {
    fn from(error: core_money::ConversionError) -> Self {
        Self::ClientError(Box::new(error))
    }
}
