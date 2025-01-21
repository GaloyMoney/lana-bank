use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrialBalanceStatementLedgerError {
    #[error("TrialBalanceStatementLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("TrialBalanceStatementLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("TrialBalanceStatementLedgerError - CalaAccountSetError: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
}
