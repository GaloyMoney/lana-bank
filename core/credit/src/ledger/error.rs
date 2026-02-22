use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

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
    ConversionError(#[from] money::ConversionError),
    #[error("CreditLedgerError - CalaVelocityError: {0}")]
    CalaVelocity(#[from] cala_ledger::velocity::error::VelocityError),
    #[error("CreditLedgerError - ChartLookupError: {0}")]
    ChartLookupError(#[from] core_accounting_primitives::ChartLookupError),
    #[error(
        "CreditLedgerError - NonAccountMemberFoundInAccountSet: Found non-Account typed member in account set {0}"
    )]
    NonAccountMemberFoundInAccountSet(String),
    #[error("CreditLedgerError - JournalIdMismatch: Account sets have wrong JournalId")]
    JournalIdMismatch,
    #[error("CreditLedgerError - PaymentAmountGreaterThanOutstandingObligations")]
    PaymentAmountGreaterThanOutstandingObligations,
}

impl ErrorSeverity for CreditLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaAccount(_) => Level::ERROR,
            Self::AccountSetError(_) => Level::ERROR,
            Self::CalaTxTemplate(_) => Level::ERROR,
            Self::CalaBalance(_) => Level::ERROR,
            Self::ConversionError(e) => e.severity(),
            Self::CalaVelocity(_) => Level::ERROR,
            Self::ChartLookupError(e) => e.severity(),
            Self::NonAccountMemberFoundInAccountSet(_) => Level::ERROR,
            Self::JournalIdMismatch => Level::ERROR,
            Self::PaymentAmountGreaterThanOutstandingObligations => Level::WARN,
        }
    }
}
