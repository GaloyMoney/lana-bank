use thiserror::Error;

use crate::primitives::TermsTemplateId;

#[derive(Error, Debug)]
pub enum TermsTemplateError {
    #[error("TermsTemplateError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("TermsTemplateError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("TermsTemplateError - CouldNotFindById: {0}")]
    CouldNotFindById(TermsTemplateId),
    #[error("TermsTemplateError - NotFound")]
    NotFound,
    #[error("TermsTemplateError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("TermsTemplateError - AuditError: {0}")]
    AuditError(#[from] crate::audit::error::AuditError),
    #[error("TermsTemplateError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
}

impl From<es_entity::EsEntityError> for TermsTemplateError {
    fn from(e: es_entity::EsEntityError) -> Self {
        match e {
            es_entity::EsEntityError::NotFound => TermsTemplateError::NotFound,
            es_entity::EsEntityError::UninitializedFieldError(e) => {
                panic!(
                    "Inconsistent data when initializing a TermsTemplate entity: {:?}",
                    e
                )
            }
        }
    }
}
