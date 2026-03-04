use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{ReportCreateError, ReportFindError, ReportModifyError, ReportQueryError};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum ReportError {
    #[error("ReportError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ReportError - Create: {0}")]
    Create(#[from] ReportCreateError),
    #[error("ReportError - Modify: {0}")]
    Modify(#[from] ReportModifyError),
    #[error("ReportError - Find: {0}")]
    Find(#[from] ReportFindError),
    #[error("ReportError - Query: {0}")]
    Query(#[from] ReportQueryError),
    #[error("ReportError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ReportError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
}

impl ErrorSeverity for ReportError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
        }
    }
}
