use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerAccountError {
    #[error("LedgerAccountError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("LedgerAccountError - LedgerAccountLedgerError: {0}")]
    LedgerAccountLedgerError(#[from] super::ledger::error::LedgerAccountLedgerError),
}
