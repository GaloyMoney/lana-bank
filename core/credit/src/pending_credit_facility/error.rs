use thiserror::Error;

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
    #[error("PendingCreditFacilityError - PriceListenerError: {0}")]
    PriceListenerError(#[from] core_price_listener::error::PriceListenerError),
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
}

es_entity::from_es_entity_error!(PendingCreditFacilityError);
