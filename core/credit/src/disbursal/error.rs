use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    DisbursalCreateError, DisbursalFindError, DisbursalModifyError, DisbursalQueryError,
};

#[derive(Error, Debug)]
pub enum DisbursalError {
    #[error("DisbursalError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DisbursalError - Create: {0}")]
    Create(#[from] DisbursalCreateError),
    #[error("DisbursalError - Modify: {0}")]
    Modify(#[from] DisbursalModifyError),
    #[error("DisbursalError - Find: {0}")]
    Find(#[from] DisbursalFindError),
    #[error("DisbursalError - Query: {0}")]
    Query(#[from] DisbursalQueryError),
    #[error("DisbursalError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("DisbursalError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("DisbursalError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("DisbursalError - ObligationError: {0}")]
    ObligationError(#[from] core_credit_collection::ObligationError),
    #[error("CreditFacilityError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
}

impl ErrorSeverity for DisbursalError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::GovernanceError(e) => e.severity(),
            Self::ObligationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
        }
    }
}
