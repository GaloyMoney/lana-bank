use thiserror::Error;

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
