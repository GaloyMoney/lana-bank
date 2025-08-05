use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObligationInstallmentError {
    #[error("ObligationInstallmentError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ObligationInstallmentError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ObligationInstallmentError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ObligationInstallmentError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

es_entity::from_es_entity_error!(ObligationInstallmentError);
