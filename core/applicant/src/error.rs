use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ApplicantError {
    #[error("ApplicantError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ApplicantError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("ApplicantError - CustomerError: {0}")]
    CustomerError(#[from] core_customer::error::CustomerError),
    #[error("ApplicantError - UnhandledCallbackType")]
    UnhandledCallbackType,
    #[error("ApplicantError - UnhandledLevelType")]
    UnhandledLevelType,
    #[error("ApplicantError - MissingExternalUserId: {0}")]
    MissingExternalUserId(String),
    #[error("ApplicantError - UuidError: {0}")]
    UuidError(#[from] uuid::Error),
    #[error("ApplicantError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("ApplicantError - InboxError: {0}")]
    InboxError(#[from] obix::inbox::InboxError),
    #[error("ApplicantError - CustomerIdNotFound: {0}")]
    CustomerIdNotFound(String),
    #[error("ApplicantError - SumsubVerificationLevelParseError: Could not parse '{0}'")]
    SumsubVerificationLevelParseError(String),
    #[error("ApplicantError - ReviewAnswerParseError: Could not parse '{0}'")]
    ReviewAnswerParseError(String),
    #[error("ApplicantError - SumsubError: {0}")]
    SumsubError(#[from] sumsub::SumsubError),
    #[error("ApplicantError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

impl ErrorSeverity for ApplicantError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Serde(_) => Level::ERROR,
            Self::CustomerError(e) => e.severity(),
            Self::UnhandledCallbackType => Level::ERROR,
            Self::UnhandledLevelType => Level::ERROR,
            Self::MissingExternalUserId(_) => Level::WARN,
            Self::UuidError(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::InboxError(_) => Level::ERROR,
            Self::CustomerIdNotFound(_) => Level::WARN,
            Self::SumsubVerificationLevelParseError(_) => Level::ERROR,
            Self::ReviewAnswerParseError(_) => Level::ERROR,
            Self::SumsubError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
        }
    }
}
