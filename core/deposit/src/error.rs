use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreDepositError {
    #[error("CoreDepositError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreDepositError - AuditError: {0}")]
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
    CoreChartOfAccountsError(
        #[from] core_accounting::chart_of_accounts::error::ChartOfAccountsError,
    ),
    #[error("CoreDepositError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CoreDepositError - ProcessError: {0}")]
    ProcessError(#[from] crate::processes::error::ProcessError),
    #[error("CoreDepositError - SubjectIsNotDepositAccountHolder")]
    SubjectIsNotDepositAccountHolder,
    #[error("CoreDepositError - DepositAccountNotFound")]
    DepositAccountNotFound,
    #[error("CoreDepositError - ChartIdMismatch")]
    ChartIdMismatch,
    #[error("CoreDepositError - DepositConfigAlreadyExists")]
    DepositConfigAlreadyExists,
    #[error("CoreDepositError - DepositAccountNotActive")]
    DepositAccountNotActive,
}

impl CoreDepositError {
    pub fn is_account_already_exists(&self) -> bool {
        matches!(
            self,
            Self::CoreChartOfAccountsError(
                core_accounting::chart_of_accounts::error::ChartOfAccountsError::CalaAccount(
                    cala_ledger::account::error::AccountError::ExternalIdAlreadyExists
                )
            )
        )
    }
}
