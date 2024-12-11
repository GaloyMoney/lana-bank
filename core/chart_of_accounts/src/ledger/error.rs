use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChartOfAccountsLedgerError {
    #[error("ChartOfAccountsLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ChartOfAccountsLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("ChartOfAccountsLedgerError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("ChartOfAccountsLedgerError - CalaBalanceError: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("ChartOfAccountsLedgerError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
}
