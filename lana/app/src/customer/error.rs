use thiserror::Error;

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
    #[error("CustomerError - KratosClientError: {0}")]
    KratosClientError(#[from] super::kratos::error::KratosClientError),
    #[error("CustomerError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("CustomerError - AuditError: ${0}")]
    AuditError(#[from] crate::audit::error::AuditError),
    #[error("CustomerError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("CustomerError - DepositError: {0}")]
    DepositError(#[from] crate::deposit::error::CoreDepositError),
}

es_entity::from_es_entity_error!(CustomerError);
