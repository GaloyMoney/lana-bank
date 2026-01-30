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
    #[error("ChartOfAccountsIntegrationError - AccountingBaseConfigNotFound")]
    AccountingBaseConfigNotFound,
    #[error("ChartOfAccountsIntegrationError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::error::DomainConfigError),
    #[error("ChartOfAccountsIntegrationError - EsEntityError: {0}")]
    EsEntityError(#[from] es_entity::EsEntityError),
    #[error("ChartOfAccountsIntegrationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl ErrorSeverity for ChartOfAccountsIntegrationError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartIdMismatch => Level::ERROR,
            Self::DepositLedgerError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::AccountingBaseConfigNotFound => Level::ERROR,
            Self::DomainConfigError(e) => e.severity(),
            Self::EsEntityError(e) => e.severity(),
            Self::Sqlx(_) => Level::ERROR,
        }
    }
}
