use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ProfitAndLossStatementLedgerError {
    #[error("ProfitAndLossStatementLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ProfitAndLossStatementLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("ProfitAndLossStatementLedgerError - CalaAccountSet: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("ProfitAndLossStatementLedgerError - CalaBalance: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("ProfitAndLossStatementLedgerError - NonAccountSetMemberTypeFound")]
    NonAccountSetMemberTypeFound,
    #[error("ProfitAndLossStatementLedgerError - NotFound: {0}")]
    NotFound(String),
}

impl ProfitAndLossStatementLedgerError {
    pub fn account_set_exists(&self) -> bool {
        matches!(
            self,
            Self::CalaAccountSet(
                cala_ledger::account_set::error::AccountSetError::ExternalIdAlreadyExists,
            )
        )
    }
}

impl ErrorSeverity for ProfitAndLossStatementLedgerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaAccountSet(
                cala_ledger::account_set::error::AccountSetError::ExternalIdAlreadyExists,
            ) => Level::WARN,
            Self::CalaAccountSet(_) => Level::ERROR,
            Self::CalaBalance(_) => Level::ERROR,
            Self::NonAccountSetMemberTypeFound => Level::ERROR,
            Self::NotFound(_) => Level::WARN,
        }
    }
}
