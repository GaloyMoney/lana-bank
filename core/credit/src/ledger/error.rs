use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CreditLedgerError {
    #[error("CreditLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditLedgerError - Ledger: {0}")]
    Ledger(Box<dyn std::error::Error + Send + Sync>),
    #[error("CreditLedgerError - ConversionError: {0}")]
    ConversionError(#[from] money::ConversionError),
    #[error("CreditLedgerError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] core_accounting::chart_of_accounts::error::ChartOfAccountsError),
    #[error(
        "CreditLedgerError - NonAccountMemberFoundInAccountSet: Found non-Account typed member in account set {0}"
    )]
    NonAccountMemberFoundInAccountSet(String),
    #[error("CreditLedgerError - JournalIdMismatch: Account sets have wrong JournalId")]
    JournalIdMismatch,
    #[error("CreditLedgerError - PaymentAmountGreaterThanOutstandingObligations")]
    PaymentAmountGreaterThanOutstandingObligations,
}

impl CreditLedgerError {
    pub fn from_ledger(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Ledger(Box::new(e))
    }
}

impl ErrorSeverity for CreditLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Ledger(_) => Level::ERROR,
            Self::ConversionError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::NonAccountMemberFoundInAccountSet(_) => Level::ERROR,
            Self::JournalIdMismatch => Level::ERROR,
            Self::PaymentAmountGreaterThanOutstandingObligations => Level::WARN,
        }
    }
}
