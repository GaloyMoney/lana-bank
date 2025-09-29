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
    #[error("PendingCreditFacilityError - PriceError: {0}")]
    PriceError(#[from] core_price::error::PriceError),
    #[error("PendingCreditFacilityError - ApprovalInProgress")]
    ApprovalInProgress,
    #[error("PendingCreditFacilityError - BelowMarginLimit")]
    BelowMarginLimit,
    #[error("PendingCreditFacilityError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

es_entity::from_es_entity_error!(PendingCreditFacilityError);
