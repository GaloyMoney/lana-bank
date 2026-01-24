use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::error::CustomerError;

#[derive(Error, Debug)]
pub enum KycError {
    #[error("KycError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("KycError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("KycError - CustomerError: {0}")]
    CustomerError(#[from] CustomerError),
    #[error("KycError - UnhandledCallbackType")]
    UnhandledCallbackType,
    #[error("KycError - UnhandledLevelType")]
    UnhandledLevelType,
    #[error("KycError - MissingExternalUserId: {0}")]
    MissingExternalUserId(String),
    #[error("KycError - InboxError: {0}")]
    InboxError(#[from] obix::inbox::InboxError),
    #[error("KycError - KycLevelParseError: Could not parse '{0}'")]
    KycLevelParseError(String),
    #[error("KycError - ReviewAnswerParseError: Could not parse '{0}'")]
    ReviewAnswerParseError(String),
    #[error("KycError - SumsubError: {0}")]
    SumsubError(#[from] sumsub::SumsubError),
    #[error("KycError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

impl ErrorSeverity for KycError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Serde(_) => Level::ERROR,
            Self::CustomerError(e) => e.severity(),
            Self::UnhandledCallbackType => Level::ERROR,
            Self::UnhandledLevelType => Level::ERROR,
            Self::MissingExternalUserId(_) => Level::WARN,
            Self::InboxError(_) => Level::ERROR,
            Self::KycLevelParseError(_) => Level::ERROR,
            Self::ReviewAnswerParseError(_) => Level::ERROR,
            Self::SumsubError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
        }
    }
}
