use thiserror::Error;

#[derive(Error, Debug)]
pub enum BalanceSheetError {
    #[error("BalanceSheetError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("BalanceSheetError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("BalanceSheetError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("BalanceSheetError - BalanceSheetLedgerError: {0}")]
    BalanceSheetLedgerError(#[from] super::ledger::error::BalanceSheetLedgerError),
}
