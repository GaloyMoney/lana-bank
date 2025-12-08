use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("ProcessError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ProcessError - Governance: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("ProcessError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ProcessError - WithdrawalError: {0}")]
    WithdrawalError(#[from] crate::withdrawal::error::WithdrawalError),
    #[error("ProcessError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
}

es_entity::from_es_entity_error!(ProcessError);

impl ErrorSeverity for ProcessError {
    fn severity(&self) -> Level {
        match self {
            Self::EsEntityError(e) => e.severity(),
            Self::GovernanceError(e) => e.severity(),
            Self::Sqlx(_) => Level::ERROR,
            Self::WithdrawalError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
        }
    }
}
