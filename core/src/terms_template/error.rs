use crate::primitives::LoanTermsId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TermsTemplateError {
    #[error("TermsTemplateError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("TermsTemplateError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("TermsTemplateError - CouldNotFindById: {0}")]
    CouldNotFindById(LoanTermsId),
    #[error("TermsTemplateError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("TermsTemplateError - AuditError: {0}")]
    AuditError(#[from] crate::audit::error::AuditError),
    #[error("TermsTemplateError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
}
