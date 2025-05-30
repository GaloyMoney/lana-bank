use thiserror::Error;

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
}
