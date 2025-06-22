use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoanAgreementError {
    #[error("LoanAgreementError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LoanAgreementError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("LoanAgreementError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("LoanAgreementError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("LoanAgreementError - StorageError: {0}")]
    StorageError(#[from] cloud_storage::error::StorageError),
    #[error("LoanAgreementError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("LoanAgreementError - GenerationError: {0}")]
    GenerationError(String),
    #[error("LoanAgreementError - LoanAgreementNotReady")]
    LoanAgreementNotReady,
    #[error("LoanAgreementError - LoanAgreementFileNotFound")]
    LoanAgreementFileNotFound,
    #[error("LoanAgreementError - TemplateRenderingError: {0}")]
    TemplateRenderingError(#[from] handlebars::RenderError),
}

es_entity::from_es_entity_error!(LoanAgreementError);