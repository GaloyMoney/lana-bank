use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ChartOfAccountsIntegrationError {
    #[error("ChartOfAccountsIntegrationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ChartOfAccountsIntegrationError - ChartIdMismatch")]
    ChartIdMismatch,
    #[error("ChartOfAccountsIntegrationError - ConfigAlreadySet")]
    ConfigAlreadySet,
    #[error("ChartOfAccountsIntegrationError - CreditLedgerError: {0}")]
    CreditLedgerError(#[from] crate::ledger::error::CreditLedgerError),
    #[error("ChartOfAccountsIntegrationError - ChartLookupError: {0}")]
    ChartLookupError(#[from] core_accounting_primitives::ChartLookupError),
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
            Self::ConfigAlreadySet => Level::WARN,
            Self::CreditLedgerError(e) => e.severity(),
            Self::ChartLookupError(e) => e.severity(),
            Self::AccountingBaseConfigNotFound => Level::ERROR,
            Self::DomainConfigError(e) => e.severity(),
            Self::EsEntityError(e) => e.severity(),
            Self::Sqlx(_) => Level::ERROR,
        }
    }
}
