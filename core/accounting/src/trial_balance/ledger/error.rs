use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum TrialBalanceLedgerError {
    #[error("TrialBalanceLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("TrialBalanceLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("TrialBalanceLedgerError - CalaAccountSet: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("TrialBalanceLedgerError - CalaBalance: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("TrialBalanceLedgerError - CalaEntry: {0}")]
    CalaEntry(#[from] cala_ledger::entry::error::EntryError),
    #[error("TrialBalanceLedgerError - NonAccountSetMemberTypeFound")]
    NonAccountSetMemberTypeFound,
    #[error("TrialBalanceLedgerError - AccountCodeParseError: {0}")]
    AccountCodeParseError(#[from] crate::primitives::AccountCodeParseError),
}

impl TrialBalanceLedgerError {
    pub fn account_set_exists(&self) -> bool {
        matches!(
            self,
            Self::CalaAccountSet(
                cala_ledger::account_set::error::AccountSetError::ExternalIdAlreadyExists,
            )
        )
    }
}

impl ErrorSeverity for TrialBalanceLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaAccountSet(_) => Level::ERROR,
            Self::CalaBalance(_) => Level::ERROR,
            Self::CalaEntry(_) => Level::ERROR,
            Self::NonAccountSetMemberTypeFound => Level::ERROR,
            Self::AccountCodeParseError(e) => e.severity(),
        }
    }
}
