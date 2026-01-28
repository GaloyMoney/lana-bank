use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum AccountingInitError {
    #[error("AccountingInitError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("AccountingInitError - JsonSerde: {0}")]
    JsonSerde(#[from] serde_json::Error),
    #[error("AccountingInitError - AccountCodeParseError: {0}")]
    AccountCodeParseError(#[from] core_accounting::AccountCodeParseError),
    #[error("AccountingInitError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] core_accounting::chart_of_accounts::error::ChartOfAccountsError),
    #[error("AccountingInitError - CreditChartOfAccountsIntegrationError: {0}")]
    CreditChartOfAccountsIntegrationError(#[from] core_credit::ChartOfAccountsIntegrationError),
    #[error("AccountingInitError - CoreDepositError: {0}")]
    CoreDepositError(#[from] core_deposit::error::CoreDepositError),
    #[error("AccountingInitError - LedgerError: {0}")]
    LedgerError(#[from] cala_ledger::error::LedgerError),
    #[error("AccountingInitError - JournalError: {0}")]
    JournalError(#[from] cala_ledger::journal::error::JournalError),
    #[error("AccountingInitError - CalaAccountError: {0}")]
    AccountError(#[from] cala_ledger::account::error::AccountError),
    #[error("AccountingInitError - TrialBalanceError: {0}")]
    TrialBalanceError(#[from] crate::trial_balance::error::TrialBalanceError),
    #[error("AccountingInitError - ProfitAndLossStatementError: {0}")]
    ProfitAndLossStatementError(#[from] crate::profit_and_loss::error::ProfitAndLossStatementError),
    #[error("AccountingInitError - BalanceSheetError: {0}")]
    BalanceSheetError(#[from] crate::balance_sheet::error::BalanceSheetError),
    #[error("AccountingInitError - FiscalYearError: {0}")]
    FiscalYearError(#[from] crate::fiscal_year::error::FiscalYearError),
    #[error("AccountingInitError - SeedFileError: {0}")]
    SeedFileError(#[from] std::io::Error),
    #[error("AccountingInitError - MissingConfig: {0}")]
    MissingConfig(String),
    #[error("AccountingInitError - AccountingBaseConfigError: {0}")]
    AccountingBaseConfigError(#[from] core_accounting::AccountingBaseConfigError),
    #[error("AccountingInitError - CoreAccountingError: {0}")]
    CoreAccountingError(#[from] core_accounting::error::CoreAccountingError),
}

impl ErrorSeverity for AccountingInitError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::JsonSerde(_) => Level::ERROR,
            Self::AccountCodeParseError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::CreditChartOfAccountsIntegrationError(e) => e.severity(),
            Self::CoreDepositError(e) => e.severity(),
            Self::LedgerError(_) => Level::ERROR,
            Self::JournalError(_) => Level::ERROR,
            Self::AccountError(_) => Level::ERROR,
            Self::TrialBalanceError(e) => e.severity(),
            Self::ProfitAndLossStatementError(e) => e.severity(),
            Self::BalanceSheetError(e) => e.severity(),
            Self::FiscalYearError(e) => e.severity(),
            Self::SeedFileError(_) => Level::ERROR,
            Self::MissingConfig(_) => Level::ERROR,
            Self::AccountingBaseConfigError(e) => e.severity(),
            Self::CoreAccountingError(e) => e.severity(),
        }
    }
}
