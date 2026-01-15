use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum BalanceSheetError {
    #[error("BalanceSheetError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("BalanceSheetError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("BalanceSheetError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("BalanceSheetError - BalanceSheetLedgerError: {0}")]
    BalanceSheetLedgerError(#[from] super::ledger::error::BalanceSheetLedgerError),
    #[error("BalanceSheetError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] crate::chart_of_accounts::error::ChartOfAccountsError),
    #[error("BalanceSheetError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::DomainConfigError),
    #[error("BalanceSheetError - BalanceSheetConfigAlreadyExists")]
    BalanceSheetConfigAlreadyExists,
    #[error("BalanceSheetError - ChartIdMismatch")]
    ChartIdMismatch,
    #[error("BalanceSheetError - AccountingBaseConfigNotFound")]
    AccountingBaseConfigNotFound,
}

impl ErrorSeverity for BalanceSheetError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::AuditError(e) => e.severity(),
            Self::AuthorizationError(e) => e.severity(),
            Self::BalanceSheetLedgerError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::DomainConfigError(e) => e.severity(),
            Self::BalanceSheetConfigAlreadyExists => Level::WARN,
            Self::ChartIdMismatch => Level::ERROR,
            Self::AccountingBaseConfigNotFound => Level::ERROR,
        }
    }
}
