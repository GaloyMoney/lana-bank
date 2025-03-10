use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreditLedgerError {
    #[error("CreditLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("CreditLedgerError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("CreditLedgerError - CalaAccountSetError: {0}")]
    AccountSetError(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("CreditLedgerError - CalaTxTemplateError: {0}")]
    CalaTxTemplate(#[from] cala_ledger::tx_template::error::TxTemplateError),
    #[error("CreditLedgerError - CalaBalanceError: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("CreditLedgerError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
    #[error("CreditLedgerError - CalaVelocityError: {0}")]
    CalaVelocity(#[from] cala_ledger::velocity::error::VelocityError),
    #[error("CreditLedgerError - CoreChartOfAccountsError: {0}")]
    CoreChartOfAccountsError(#[from] chart_of_accounts::error::CoreChartOfAccountsError),
    #[error("CreditLedgerError - NonAccountMemberFoundInAccountSet: Found non-Account typed member in account set {0}")]
    NonAccountMemberFoundInAccountSet(String),
    #[error("CreditLedgerError - JournalIdMismatch: Account sets have wrong JournalId")]
    JournalIdMismatch,
}
