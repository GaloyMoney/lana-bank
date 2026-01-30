use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ChartOfAccountsIntegrationError {
    #[error("ChartOfAccountsIntegrationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ChartOfAccountsIntegrationError - ChartIdMismatch")]
    ChartIdMismatch,
    #[error("ChartOfAccountsIntegrationError - DepositLedgerError: {0}")]
    DepositLedgerError(#[from] crate::ledger::error::DepositLedgerError),
    #[error("ChartOfAccountsIntegrationError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] core_accounting::chart_of_accounts::error::ChartOfAccountsError),
    #[error("ChartOfAccountsIntegrationError - ConfigAlreadyExists")]
    ConfigAlreadyExists,
    #[error("ChartOfAccountsIntegrationError - AccountingBaseConfigNotFound")]
    AccountingBaseConfigNotFound,
}

impl ErrorSeverity for ChartOfAccountsIntegrationError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartIdMismatch => Level::ERROR,
            Self::DepositLedgerError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::ConfigAlreadyExists => Level::WARN,
            Self::AccountingBaseConfigNotFound => Level::ERROR,
        }
    }
}
