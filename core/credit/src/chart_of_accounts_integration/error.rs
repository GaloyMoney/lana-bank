use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ChartOfAccountsIntegrationError {
    #[error("ChartOfAccountIntegrationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ChartOfAccountIntegrationError ChartIdMismatch")]
    ChartIdMismatch,
    #[error("ChartOfAccountIntegrationError - CreditLedgerError: {0}")]
    CreditLedgerError(#[from] crate::ledger::error::CreditLedgerError),
    #[error("ChartOfAccountIntegrationError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] core_accounting::chart_of_accounts::error::ChartOfAccountsError),
    #[error("ChartOfAccountIntegrationError - AccountingBaseConfigNotFound")]
    AccountingBaseConfigNotFound,
    #[error("ChartOfAccountIntegrationError - InvalidAccountingAccountSetParent: {0}")]
    InvalidAccountingAccountSetParent(String),
    #[error("ChartOfAccountIntegrationError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::error::DomainConfigError),
    #[error("ChartOfAccountIntegrationError - EsEntityError: {0}")]
    EsEntityError(#[from] es_entity::EsEntityError),
    #[error("ChartOfAccountIntegrationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl ErrorSeverity for ChartOfAccountsIntegrationError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartIdMismatch => Level::ERROR,
            Self::CreditLedgerError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::AccountingBaseConfigNotFound => Level::ERROR,
            Self::InvalidAccountingAccountSetParent(_) => Level::ERROR,
            Self::DomainConfigError(e) => e.severity(),
            Self::EsEntityError(e) => e.severity(),
            Self::Sqlx(_) => Level::ERROR,
        }
    }
}
