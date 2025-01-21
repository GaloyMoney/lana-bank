use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreStatementsError {
    #[error("CoreStatementsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreStatementsError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CoreStatementsError - TrialBalanceStatementError: {0}")]
    TrialBalanceStatementError(#[from] crate::trial_balance::error::TrialBalanceStatementError),
    #[error("CoreStatementsError - TrialBalanceStatementLedgerError: {0}")]
    TrialBalanceStatementLedgerError(
        #[from] crate::trial_balance::ledger::error::TrialBalanceStatementLedgerError,
    ),
}
