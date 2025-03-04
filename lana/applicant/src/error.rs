use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicantError {
    #[error("ApplicantError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ApplicantError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("ApplicantError - UserError: {0}")]
    CustomerError(#[from] core_customer::error::CustomerError),
    #[error("ApplicantError - SystemTimeError: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("ApplicantError - InvalidHeaderValue: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("ApplicantError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("ApplicantError - Sumsub Error: {code}, {description}")]
    Sumsub { code: u16, description: String },
    #[error("ApplicantError - UnhandledCallbackType: {0}")]
    UnhandledCallbackType(String),
    #[error("ApplicantError - MissingExternalUserId: {0}")]
    MissingExternalUserId(String),
    #[error("ApplicantError - UuidError: {0}")]
    UuidError(#[from] uuid::Error),
    #[error("ApplicantError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
}
