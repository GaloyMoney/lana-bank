use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrialBalanceLedgerError {
    #[error("TrialBalanceLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("TrialBalanceLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("TrialBalanceLedgerError - CalaAccountSetError: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
}
