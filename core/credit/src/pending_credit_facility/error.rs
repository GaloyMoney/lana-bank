use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum PendingCreditFacilityError {
    #[error("PendingCreditFacilityError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PendingCreditFacilityError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PendingCreditFacilityError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("PendingCreditFacilityError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("PendingCreditFacilityError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::CreditLedgerError),
    #[error("PendingCreditFacilityError - PriceError: {0}")]
    PriceError(#[from] core_price::error::PriceError),
    #[error("PendingCreditFacilityError - ApprovalInProgress")]
    ApprovalInProgress,
    #[error("PendingCreditFacilityError - BelowMarginLimit")]
    BelowMarginLimit,
    #[error("PendingCreditFacilityError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("PendingCreditFacilityError - CoreCustodyError: {0}")]
    CoreCustodyError(#[from] core_custody::error::CoreCustodyError),
    #[error("PendingCreditFacilityError - CollateralError: {0}")]
    CollateralError(#[from] crate::collateral::error::CollateralError),
    #[error("PendingCreditFacilityError - CreditFacilityProposalError: {0}")]
    CreditFacilityProposalError(
        #[from] crate::credit_facility_proposal::error::CreditFacilityProposalError,
    ),
    #[error("PendingCreditFacilityError - AuditError: ${0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CoreCreditError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
}

impl ErrorSeverity for PendingCreditFacilityError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::GovernanceError(e) => e.severity(),
            Self::LedgerError(e) => e.severity(),
            Self::PriceError(e) => e.severity(),
            Self::ApprovalInProgress => Level::WARN,
            Self::BelowMarginLimit => Level::WARN,
            Self::AuthorizationError(e) => e.severity(),
            Self::CoreCustodyError(e) => e.severity(),
            Self::CollateralError(e) => e.severity(),
            Self::CreditFacilityProposalError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
        }
    }
}

es_entity::from_es_entity_error!(PendingCreditFacilityError);
