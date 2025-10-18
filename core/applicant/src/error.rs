use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicantError {
    #[error("ApplicantError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ApplicantError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("ApplicantError - CustomerError: {0}")]
    CustomerError(#[from] core_customer::error::CustomerError),
    #[error("ApplicantError - SystemTimeError: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
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
