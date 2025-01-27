use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProfitAndLossStatementError {
    #[error("ProfitAndLossStatementError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ProfitAndLossStatementError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("ProfitAndLossStatementError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ProfitAndLossStatementError - ProfitAndLossStatementLedgerError: {0}")]
    ProfitAndLossStatementLedgerError(
        #[from] super::ledger::error::ProfitAndLossStatementLedgerError,
    ),
    #[error("ProfitAndLossStatementError - MultipleFound: {0}")]
    MultipleFound(String),
    #[error("ProfitAndLossStatementError - NotFound: {0}")]
    NotFound(String),
}
