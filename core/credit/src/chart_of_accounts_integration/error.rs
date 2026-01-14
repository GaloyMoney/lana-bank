use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ChartOfAccountsIntegrationError {
    #[error("ChartOfAccountIntegrationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ChartOfAccountIntegrationError ChartIdMismatch")]
    ChartIdMismatch,
    #[error("ChartOfAccountIntegrationError - CreditConfigAlreadyExists")]
    CreditConfigAlreadyExists,
    #[error("ChartOfAccountIntegrationError - CreditLedgerError: {0}")]
    CreditLedgerError(#[from] crate::ledger::error::CreditLedgerError),
    #[error("ChartOfAccountIntegrationError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] core_accounting::chart_of_accounts::error::ChartOfAccountsError),
    #[error("ChartOfAccountIntegrationError - InvalidAccountingAccountSetParent: {0}")]
    InvalidAccountingAccountSetParent(String),
    #[error("ChartOfAccountsIntegrationError - AccountingBaseConfigNotFound")]
    AccountingBaseConfigNotFound,
}

impl ErrorSeverity for ChartOfAccountsIntegrationError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartIdMismatch => Level::ERROR,
            Self::CreditConfigAlreadyExists => Level::WARN,
            Self::CreditLedgerError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::InvalidAccountingAccountSetParent(_) => Level::ERROR,
            Self::AccountingBaseConfigNotFound => Level::ERROR,
        }
    }
}
