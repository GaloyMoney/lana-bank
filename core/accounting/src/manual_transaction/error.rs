use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::chart_of_accounts;

use super::repo::{
    ManualTransactionCreateError, ManualTransactionFindError, ManualTransactionModifyError,
    ManualTransactionQueryError,
};

#[derive(Error, Debug)]
pub enum ManualTransactionError {
    #[error("ManualTransactionError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ManualTransactionError - Create: {0}")]
    Create(#[from] ManualTransactionCreateError),
    #[error("ManualTransactionError - Modify: {0}")]
    Modify(#[from] ManualTransactionModifyError),
    #[error("ManualTransactionError - Find: {0}")]
    Find(#[from] ManualTransactionFindError),
    #[error("ManualTransactionError - Query: {0}")]
    Query(#[from] ManualTransactionQueryError),
    #[error("ManualTransactionError - ManualTransactionLedgerError: {0}")]
    ManualTransactionLedgerError(#[from] super::ledger::error::ManualTransactionLedgerError),
    #[error("ManualTransactionError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ManualTransactionError - ChartOfAccounts: {0}")]
    ChartOfAccountsError(#[from] chart_of_accounts::error::ChartOfAccountsError),
}

impl ErrorSeverity for ManualTransactionError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::ManualTransactionLedgerError(e) => e.severity(),
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
        }
    }
}
