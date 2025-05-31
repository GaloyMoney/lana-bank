use thiserror::Error;

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
    #[error("AccountingInitError - CreditChartIntegrationConfigBuilderError: {0}")]
    CreditChartIntegrationConfigBuilderError(
        #[from] core_credit::ChartOfAccountsIntegrationConfigBuilderError,
    ),
    #[error("AccountingInitError - DepositChartIntegrationConfigBuilderError: {0}")]
    DepositChartIntegrationConfigBuilderError(
        #[from] core_deposit::ChartOfAccountsIntegrationConfigBuilderError,
    ),
    #[error("AccountingInitError - BalanceSheetChartIntegrationConfigBuilderError: {0}")]
    BalanceSheetChartIntegrationConfigBuilderError(
        #[from] crate::balance_sheet::ChartOfAccountsIntegrationConfigBuilderError,
    ),
    #[error("AccountingInitError - ProfitAndLossStatementChartIntegrationConfigBuilderError: {0}")]
    ProfitAndLossStatementChartIntegrationConfigBuilderError(
        #[from] crate::profit_and_loss::ChartOfAccountsIntegrationConfigBuilderError,
    ),
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
    #[error("AccountingInitError - SeedFileError: {0}")]
    SeedFileError(#[from] std::io::Error),
}
