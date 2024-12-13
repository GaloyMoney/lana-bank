use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositLedgerError {
    #[error("DepositLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("DepositLedgerError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("DepositLedgerError - CalaTxTemplateError: {0}")]
    CalaTxTemplate(#[from] cala_ledger::tx_template::error::TxTemplateError),
    #[error("DepositLedgerError - CalaBalanceError: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("DepositLedgerError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
}
