use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum BalanceSheetLedgerError {
    #[error("BalanceSheetLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("BalanceSheetLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("BalanceSheetLedgerError - CalaAccountSet: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("BalanceSheetLedgerError - CalaBalance: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("BalanceSheetLedgerError - NonAccountSetMemberTypeFound")]
    NonAccountSetMemberTypeFound,
    #[error("BalanceSheetLedgerError - NotFound: {0}")]
    NotFound(String),
}

impl BalanceSheetLedgerError {
    pub fn account_set_exists(&self) -> bool {
        matches!(
            self,
            Self::CalaAccountSet(
                cala_ledger::account_set::error::AccountSetError::ExternalIdAlreadyExists,
            )
        )
    }
}

impl ErrorSeverity for BalanceSheetLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaAccountSet(_) => {
                if self.account_set_exists() {
                    Level::WARN
                } else {
                    Level::ERROR
                }
            }
            Self::CalaBalance(_) => Level::ERROR,
            Self::NonAccountSetMemberTypeFound => Level::ERROR,
            Self::NotFound(_) => Level::WARN,
        }
    }
}
