use thiserror::Error;

use crate::primitives::DepositAccountHolderId;

#[derive(Error, Debug)]
pub enum CoreDepositError {
    #[error("CoreDepositError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreChartOfAccountsError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CoreDepositError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreDepositError - DepositAccountError: {0}")]
    DepositAccountError(#[from] crate::account::error::DepositAccountError),
    #[error("CoreDepositError - DepositError: {0}")]
    DepositError(#[from] crate::deposit::error::DepositError),
    #[error("CoreDepositError - WithdrawalError: {0}")]
    WithdrawalError(#[from] crate::withdrawal::error::WithdrawalError),
    #[error("CoreDepositError - DepositLedgerError: {0}")]
    DepositLedgerError(#[from] crate::ledger::error::DepositLedgerError),
    #[error("CoreDepositError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("CoreDepositError - CoreChartOfAccountsError: {0}")]
    CoreChartOfAccountsError(#[from] chart_of_accounts::error::CoreChartOfAccountsError),
    #[error("CoreDepositError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CoreDepositError - ProcessError: {0}")]
    ProcessError(#[from] crate::processes::error::ProcessError),
    #[error("CoreDepositError - MultipleFoundForAccountHolder: {0}")]
    MultipleFoundForAccountHolder(DepositAccountHolderId),
}
