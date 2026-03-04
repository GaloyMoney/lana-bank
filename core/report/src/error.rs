use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum ReportError {
    #[error("ReportError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ReportError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ReportError - AuditError: ${0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("ReportError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("ReportError - DagsterError: {0}")]
    DagsterError(#[from] dagster::error::DagsterError),
    #[error("ReportError - StorageError: {0}")]
    StorageError(#[from] cloud_storage::error::StorageError),
    #[error("ReportError - ReportError: {0}")]
    ReportError(#[from] crate::report::error::ReportError),
    #[error("ReportError - ReportRunError: {0}")]
    ReportRunError(#[from] crate::report_run::error::ReportRunError),
    #[error("ReportError - NotFound")]
    NotFound,
}

impl From<crate::report::ReportFindError> for ReportError {
    fn from(e: crate::report::ReportFindError) -> Self {
        ReportError::ReportError(crate::report::error::ReportError::from(e))
    }
}

impl From<crate::report::ReportQueryError> for ReportError {
    fn from(e: crate::report::ReportQueryError) -> Self {
        ReportError::ReportError(crate::report::error::ReportError::from(e))
    }
}

impl From<crate::report_run::ReportRunFindError> for ReportError {
    fn from(e: crate::report_run::ReportRunFindError) -> Self {
        ReportError::ReportRunError(crate::report_run::error::ReportRunError::from(e))
    }
}

impl From<crate::report_run::ReportRunQueryError> for ReportError {
    fn from(e: crate::report_run::ReportRunQueryError) -> Self {
        ReportError::ReportRunError(crate::report_run::error::ReportRunError::from(e))
    }
}

impl ErrorSeverity for ReportError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
            Self::StorageError(e) => e.severity(),
            Self::ReportError(e) => e.severity(),
            Self::ReportRunError(e) => e.severity(),
            Self::DagsterError(e) => e.severity(),
            Self::NotFound => Level::WARN,
        }
    }

    fn variant_name(&self) -> &'static str {
        self.into()
    }
}
