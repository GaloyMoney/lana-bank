use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    CreditFacilityProposalCreateError, CreditFacilityProposalFindError,
    CreditFacilityProposalModifyError, CreditFacilityProposalQueryError,
};

#[derive(Error, Debug)]
pub enum CreditFacilityProposalError {
    #[error("CreditFacilityProposalError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityProposalError - Create: {0}")]
    Create(#[from] CreditFacilityProposalCreateError),
    #[error("CreditFacilityProposalError - Modify: {0}")]
    Modify(#[from] CreditFacilityProposalModifyError),
    #[error("CreditFacilityProposalError - Find: {0}")]
    Find(#[from] CreditFacilityProposalFindError),
    #[error("CreditFacilityProposalError - Query: {0}")]
    Query(#[from] CreditFacilityProposalQueryError),
    #[error("CreditFacilityProposalError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("CreditFacilityProposalError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::CreditLedgerError),
    #[error("CreditFacilityProposalError - PriceError: {0}")]
    PriceError(#[from] core_price::error::PriceError),
    #[error("CreditFacilityProposalError - ApprovalInProgress")]
    ApprovalInProgress,
    #[error("CreditFacilityProposalError - BelowMarginLimit")]
    BelowMarginLimit,
    #[error("CreditFacilityProposalError - ApprovalProcessNotStarted")]
    ApprovalProcessNotStarted,
    #[error("CreditFacilityProposalError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CreditFacilityProposalError - AuditError: ${0}")]
    AuditError(#[from] audit::error::AuditError),
}

impl ErrorSeverity for CreditFacilityProposalError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::GovernanceError(e) => e.severity(),
            Self::LedgerError(e) => e.severity(),
            Self::PriceError(e) => e.severity(),
            Self::ApprovalInProgress => Level::WARN,
            Self::BelowMarginLimit => Level::WARN,
            Self::ApprovalProcessNotStarted => Level::WARN,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
        }
    }
}
