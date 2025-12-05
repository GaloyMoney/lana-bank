use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CustomerError {
    #[error("CustomerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustomerError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CustomerError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CustomerError - UnexpectedCurrency")]
    UnexpectedCurrency,
    #[error("CustomerError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CustomerError - AuditError: ${0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CustomerError - SubjectIsNotCustomer")]
    SubjectIsNotCustomer,
    #[error("CustomerError - DocumentStorageError: {0}")]
    DocumentStorageError(#[from] document_storage::error::DocumentStorageError),
    #[error("CustomerError - PublicIdError: {0}")]
    PublicIdError(#[from] public_id::PublicIdError),
}

es_entity::from_es_entity_error!(CustomerError);

impl ErrorSeverity for CustomerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::UnexpectedCurrency => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::SubjectIsNotCustomer => Level::WARN,
            Self::DocumentStorageError(e) => e.severity(),
            Self::PublicIdError(e) => e.severity(),
        }
    }
}
