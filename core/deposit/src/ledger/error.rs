use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositLedgerError {
    #[error("DepositLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("DepositLedgerError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("DepositLedgerError - CalaAccountSetError: {0}")]
    AccountSetError(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("DepositLedgerError - CalaJournalError: {0}")]
    CalaJournal(#[from] cala_ledger::journal::error::JournalError),
    #[error("DepositLedgerError - CalaTxTemplateError: {0}")]
    CalaTxTemplate(#[from] cala_ledger::tx_template::error::TxTemplateError),
    #[error("DepositLedgerError - CalaBalanceError: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("DepositLedgerError - CalaTransactionError: {0}")]
    CalaTransaction(#[from] cala_ledger::transaction::error::TransactionError),
    #[error("DepositLedgerError - CalaEntryError: {0}")]
    CalaEntry(#[from] cala_ledger::entry::error::EntryError),
    #[error("DepositLedgerError - CalaVelocityError: {0}")]
    CalaVelocity(#[from] cala_ledger::velocity::error::VelocityError),
    #[error("DepositLedgerError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
    #[error("DepositLedgerError - MissingTxMetadata")]
    MissingTxMetadata,
    #[error("DepositLedgerError - CoreChartOfAccountsError: {0}")]
    CoreChartOfAccountsError(#[from] chart_of_accounts::error::CoreChartOfAccountsError),
    #[error("DepositLedgerError - MismatchedTxMetadata: {0}")]
    MismatchedTxMetadata(serde_json::Error),
    #[error("DepositLedgerError - NonAccountMemberFoundInAccountSet: Found non-Account typed member in account set {0}")]
    NonAccountMemberFoundInAccountSet(String),
    #[error("DepositLedgerError - JournalIdMismatch: Account sets have wrong JournalId")]
    JournalIdMismatch,
}
