use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use super::repo::{
    PolicyColumn, PolicyCreateError, PolicyFindError, PolicyModifyError, PolicyQueryError,
};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum PolicyError {
    #[error("PolicyError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("PolicyError - Create: {0}")]
    Create(PolicyCreateError),
    #[error("PolicyError - Modify: {0}")]
    Modify(#[from] PolicyModifyError),
    #[error("PolicyError - Find: {0}")]
    Find(#[from] PolicyFindError),
    #[error("PolicyError - Query: {0}")]
    Query(#[from] PolicyQueryError),
    #[error("PolicyError - DuplicateApprovalProcessType")]
    DuplicateApprovalProcessType,
    #[error("PolicyError - Threshold {1} too high for committee {0}")]
    PolicyThresholdTooHigh(crate::primitives::CommitteeId, usize),
    #[error("PolicyError - Threshold {1} too low for committee {0}")]
    PolicyThresholdTooLow(crate::primitives::CommitteeId, usize),
}

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

impl From<PolicyCreateError> for PolicyError {
    fn from(error: PolicyCreateError) -> Self {
        if error.was_duplicate_by(PolicyColumn::ProcessType) {
            return Self::DuplicateApprovalProcessType;
        }
        Self::Create(error)
    }
}

impl ErrorSeverity for PolicyError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::DuplicateApprovalProcessType => Level::WARN,
            Self::PolicyThresholdTooHigh(_, _) => Level::WARN,
            Self::PolicyThresholdTooLow(_, _) => Level::WARN,
        }
    }
}
