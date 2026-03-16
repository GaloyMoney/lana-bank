use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::AccountingTemplateId;

#[derive(Error, Debug)]
pub enum AccountingTemplateError {
    #[error("AccountingTemplateError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("AccountingTemplateError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("AccountingTemplateError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("AccountingTemplateError - CouldNotFindById: {0}")]
    CouldNotFindById(AccountingTemplateId),
    #[error("AccountingTemplateError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("AccountingTemplateError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("AccountingTemplateError - DuplicateAccountingTemplateName")]
    DuplicateAccountingTemplateName,
    #[error("AccountingTemplateError - DuplicateAccountingTemplateCode")]
    DuplicateAccountingTemplateCode,
    #[error("Invalid code: {0}")]
    InvalidCode(String),
    #[error("Invalid name: {0}")]
    InvalidName(String),
    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
    #[error("Invalid entry at index {0}: {1}")]
    InvalidEntry(usize, String),
}

es_entity::from_es_entity_error!(AccountingTemplateError);

impl From<sqlx::Error> for AccountingTemplateError {
    fn from(error: sqlx::Error) -> Self {
        if let Some(db_err) = error.as_database_error()
            && let Some(constraint) = db_err.constraint()
        {
            if constraint.contains("name") {
                return Self::DuplicateAccountingTemplateName;
            }
            if constraint.contains("code") {
                return Self::DuplicateAccountingTemplateCode;
            }
        }
        Self::Sqlx(error)
    }
}

impl ErrorSeverity for AccountingTemplateError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::CouldNotFindById(_) => Level::WARN,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::DuplicateAccountingTemplateName => Level::WARN,
            Self::DuplicateAccountingTemplateCode => Level::WARN,
            Self::InvalidCode(_) => Level::WARN,
            Self::InvalidName(_) => Level::WARN,
            Self::InvalidTemplate(_) => Level::WARN,
            Self::InvalidEntry(_, _) => Level::WARN,
        }
    }
}
