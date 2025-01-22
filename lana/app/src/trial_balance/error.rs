use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrialBalanceError {
    #[error("TrialBalanceError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("TrialBalanceError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("TrialBalanceError - TrialBalanceLedgerError: {0}")]
    TrialBalanceLedgerError(#[from] super::ledger::error::TrialBalanceLedgerError),
    #[error("TrialBalanceError - MultipleFound: {0}")]
    MultipleFound(String),
}
