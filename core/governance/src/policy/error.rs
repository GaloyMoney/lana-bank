use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("PolicyError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("PolicyError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PolicyError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("PolicyError - DuplicateApprovalProcessType")]
    DuplicateApprovalProcessType,
    #[error("PolicyError - Threshold {1} too high for committee {0}")]
    PolicyThresholdTooHigh(crate::primitives::CommitteeId, usize),
    #[error("PolicyError - Threshold {1} too low for committee {0}")]
    PolicyThresholdTooLow(crate::primitives::CommitteeId, usize),
}

es_entity::from_es_entity_error!(PolicyError);

impl From<sqlx::Error> for PolicyError {
    fn from(error: sqlx::Error) -> Self {
        if let Some(err) = error.as_database_error()
            && let Some(constraint) = err.constraint()
            && constraint.contains("type")
        {
            return Self::DuplicateApprovalProcessType;
        }
        Self::Sqlx(error)
    }
}

impl ErrorSeverity for PolicyError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(_) => Level::ERROR,
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::DuplicateApprovalProcessType => Level::WARN,
            Self::PolicyThresholdTooHigh(_, _) => Level::WARN,
            Self::PolicyThresholdTooLow(_, _) => Level::WARN,
        }
    }
}
