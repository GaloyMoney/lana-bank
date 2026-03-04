use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::TermsTemplateId;
use super::repo::{
    TermsTemplateColumn, TermsTemplateCreateError, TermsTemplateFindError,
    TermsTemplateModifyError, TermsTemplateQueryError,
};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum TermsTemplateError {
    #[error("TermsTemplateError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("TermsTemplateError - Create: {0}")]
    Create(TermsTemplateCreateError),
    #[error("TermsTemplateError - Modify: {0}")]
    Modify(#[from] TermsTemplateModifyError),
    #[error("TermsTemplateError - Find: {0}")]
    Find(#[from] TermsTemplateFindError),
    #[error("TermsTemplateError - Query: {0}")]
    Query(#[from] TermsTemplateQueryError),
    #[error("TermsTemplateError - CouldNotFindById: {0}")]
    CouldNotFindById(TermsTemplateId),
    #[error("TermsTemplateError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("TermsTemplateError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("TermsTemplateError - DuplicateTermsTemplateName")]
    DuplicateTermsTemplateName,
}

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

impl From<TermsTemplateCreateError> for TermsTemplateError {
    fn from(error: TermsTemplateCreateError) -> Self {
        if error.was_duplicate_by(TermsTemplateColumn::Name) {
            return Self::DuplicateTermsTemplateName;
        }
        Self::Create(error)
    }
}

impl ErrorSeverity for TermsTemplateError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::CouldNotFindById(_) => Level::WARN,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::DuplicateTermsTemplateName => Level::WARN,
        }
    }

    fn variant_name(&self) -> &'static str {
        self.into()
    }
}
