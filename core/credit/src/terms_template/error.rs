use thiserror::Error;

use crate::primitives::TermsTemplateId;

#[derive(Error, Debug)]
pub enum TermsTemplateError {
    #[error("TermsTemplateError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("TermsTemplateError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("TermsTemplateError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("TermsTemplateError - CouldNotFindById: {0}")]
    CouldNotFindById(TermsTemplateId),
    #[error("TermsTemplateError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("TermsTemplateError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
}

es_entity::from_es_entity_error!(TermsTemplateError);
