use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    ProspectCreateError, ProspectFindError, ProspectModifyError, ProspectQueryError,
};

#[derive(Error, Debug)]
pub enum ProspectError {
    #[error("ProspectError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ProspectError - Create: {0}")]
    Create(#[from] ProspectCreateError),
    #[error("ProspectError - Modify: {0}")]
    Modify(#[from] ProspectModifyError),
    #[error("ProspectError - Find: {0}")]
    Find(#[from] ProspectFindError),
    #[error("ProspectError - Query: {0}")]
    Query(#[from] ProspectQueryError),
    #[error("ProspectError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ProspectError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("ProspectError - PublicIdError: {0}")]
    PublicIdError(#[from] public_id::PublicIdError),
    #[error("ProspectError - ApplicantIdMismatch: expected {expected:?}, got {actual}")]
    ApplicantIdMismatch {
        expected: Option<String>,
        actual: String,
    },
    #[error(
        "ProspectError - KycNotStarted: cannot approve or decline KYC before it has been started"
    )]
    KycNotStarted,
    #[error("ProspectError - AlreadyConverted: prospect has already been converted to a customer")]
    AlreadyConverted,
    #[error("ProspectError - AlreadyClosed: prospect has been closed")]
    AlreadyClosed,
    #[error("ProspectError - ManualConversionNotAllowed")]
    ManualConversionNotAllowed,
}

impl ErrorSeverity for ProspectError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::PublicIdError(e) => e.severity(),
            Self::ApplicantIdMismatch { .. } => Level::WARN,
            Self::KycNotStarted => Level::WARN,
            Self::AlreadyConverted => Level::WARN,
            Self::AlreadyClosed => Level::WARN,
            Self::ManualConversionNotAllowed => Level::WARN,
        }
    }
}
