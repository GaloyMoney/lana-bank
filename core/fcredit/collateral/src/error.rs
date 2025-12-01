use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreCreditCollateralError {
    #[error("CoreCreditCollateralError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreCreditCollateralError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CoreCreditCollateralError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CoreCreditCollateralError - CreditError: {0}")]
    CreditLedgerError(#[from] crate::ledger::error::CreditLedgerError),
}
