use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use super::repo::{
    PolicyColumn, PolicyCreateError, PolicyFindError, PolicyModifyError, PolicyQueryError,
};

#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("PolicyError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
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
    #[error(
        "PolicyError - AutoApproveNotAllowed: cannot create or update policy with SystemAutoApprove when RequireCommitteeApproval is enabled"
    )]
    AutoApproveNotAllowed,
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
            Self::AutoApproveNotAllowed => Level::WARN,
        }
    }
}
