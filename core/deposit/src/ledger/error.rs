use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

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
    ConversionError(#[from] money::ConversionError),
    #[error("DepositLedgerError - MissingTxMetadata")]
    MissingTxMetadata,
    #[error("DepositLedgerError - MismatchedTxMetadata: {0}")]
    MismatchedTxMetadata(serde_json::Error),
    #[error(
        "DepositLedgerError - NonAccountMemberFoundInAccountSet: Found non-Account typed member in account set {0}"
    )]
    NonAccountMemberFoundInAccountSet(String),
    #[error("DepositLedgerError - JournalIdMismatch: Account sets have wrong JournalId")]
    JournalIdMismatch,
}

impl ErrorSeverity for DepositLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaAccount(_) => Level::ERROR,
            Self::AccountSetError(_) => Level::ERROR,
            Self::CalaJournal(_) => Level::ERROR,
            Self::CalaTxTemplate(_) => Level::ERROR,
            Self::CalaBalance(_) => Level::ERROR,
            Self::CalaTransaction(_) => Level::ERROR,
            Self::CalaEntry(_) => Level::ERROR,
            Self::CalaVelocity(_) => Level::ERROR,
            Self::ConversionError(e) => e.severity(),
            Self::MissingTxMetadata => Level::WARN,
            Self::MismatchedTxMetadata(_) => Level::WARN,
            Self::NonAccountMemberFoundInAccountSet(_) => Level::ERROR,
            Self::JournalIdMismatch => Level::ERROR,
        }
    }
}
