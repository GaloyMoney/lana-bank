use thiserror::Error;

#[derive(Error, Debug)]
pub enum CashFlowStatementError {
    #[error("CashFlowStatementError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CashFlowStatementError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CashFlowStatementError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CashFlowStatementError - CashFlowStatementLedgerError: {0}")]
    CashFlowStatementLedgerError(#[from] super::ledger::error::CashFlowStatementLedgerError),
}
