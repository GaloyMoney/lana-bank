use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrialBalanceLedgerError {
    #[error("TrialBalanceLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("TrialBalanceLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("TrialBalanceLedgerError - CalaAccountSet: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("TrialBalanceLedgerError - CalaBalance: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("TrialBalanceLedgerError - Statement: {0}")]
    Statement(#[from] crate::statement::error::StatementError),
    #[error("TrialBalanceLedgerError - NonAccountSetMemberTypeFound")]
    NonAccountSetMemberTypeFound,
}
