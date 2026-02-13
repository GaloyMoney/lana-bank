use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ProspectError {
    #[error("ProspectError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ProspectError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ProspectError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
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
}

es_entity::from_es_entity_error!(ProspectError);

impl ErrorSeverity for ProspectError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::PublicIdError(e) => e.severity(),
            Self::ApplicantIdMismatch { .. } => Level::WARN,
            Self::KycNotStarted => Level::WARN,
        }
    }
}
