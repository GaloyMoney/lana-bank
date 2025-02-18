use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountingInitError {
    #[error("AccountingInitError - CoreChartOfAccountsError: {0}")]
    CoreChartOfAccountsError(#[from] chart_of_accounts::error::CoreChartOfAccountsError),
    #[error("ApplicationError - JournalError: {0}")]
    JournalError(#[from] cala_ledger::journal::error::JournalError),
    #[error("ApplicationError - TrialBalanceError: {0}")]
    TrialBalanceError(#[from] crate::trial_balance::error::TrialBalanceError),
    #[error("ApplicationError - ProfitAndLossStatementError: {0}")]
    ProfitAndLossStatementError(#[from] crate::profit_and_loss::error::ProfitAndLossStatementError),
    #[error("ApplicationError - BalanceSheetError: {0}")]
    BalanceSheetError(#[from] crate::balance_sheet::error::BalanceSheetError),
    #[error("ApplicationError - CashFlowStatementError: {0}")]
    CashFlowStatementError(#[from] crate::cash_flow::error::CashFlowStatementError),
}
