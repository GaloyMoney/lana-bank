use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::primitives::TermsTemplateId;

#[derive(Error, Debug)]
pub enum TermsTemplateError {
    #[error("TermsTemplateError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
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
    #[error("TermsTemplateError - DuplicateTermsTemplateName")]
    DuplicateTermsTemplateName,
}

es_entity::from_es_entity_error!(TermsTemplateError);

impl From<sqlx::Error> for TermsTemplateError {
    fn from(error: sqlx::Error) -> Self {
        if let Some(db_err) = error.as_database_error()
            && let Some(constraint) = db_err.constraint()
            && constraint.contains("name")
        {
            return Self::DuplicateTermsTemplateName;
        }
        Self::Sqlx(error)
    }
}

impl ErrorSeverity for TermsTemplateError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::CouldNotFindById(_) => Level::WARN,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::DuplicateTermsTemplateName => Level::WARN,
        }
    }
}
