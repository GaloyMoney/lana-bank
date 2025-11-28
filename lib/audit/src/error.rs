use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("AuditError - Sqlx: {0} asht")]
    Sqlx(#[from] sqlx::Error),
    #[error("AuditError - SubjectParseError: Could not parse '{0}'")]
    SubjectParseError(String),
    #[error("AuditError - ObjectParseError: Could not parse '{0}'")]
    ObjectParseError(String),
    #[error("AuditError - ActionParseError: Could not parse '{0}'")]
    ActionParseError(String),
}

impl ErrorSeverity for AuditError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::SubjectParseError(_) => Level::WARN,
            Self::ObjectParseError(_) => Level::WARN,
            Self::ActionParseError(_) => Level::WARN,
        }
    }
}
