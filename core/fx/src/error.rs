use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CoreFxError {
    #[error("CoreFxError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreFxError - ChartLookupError: {0}")]
    ChartLookupError(#[from] chart_primitives::ChartLookupError),
    #[error("CoreFxError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::DomainConfigError),
    #[error("CoreFxError - FxLedgerError: {0}")]
    FxLedgerError(#[from] crate::ledger::error::FxLedgerError),
    #[error("CoreFxError - ChartOfAccountsIntegrationError: {0}")]
    ChartOfAccountsIntegrationError(
        #[from] crate::chart_of_accounts_integration::error::ChartOfAccountsIntegrationError,
    ),
}

impl ErrorSeverity for CoreFxError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartLookupError(e) => e.severity(),
            Self::DomainConfigError(e) => e.severity(),
            Self::FxLedgerError(e) => e.severity(),
            Self::ChartOfAccountsIntegrationError(e) => e.severity(),
        }
    }
}
