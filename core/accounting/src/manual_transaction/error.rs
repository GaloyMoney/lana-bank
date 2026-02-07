use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::chart_of_accounts;

#[derive(Error, Debug)]
pub enum ManualTransactionError {
    #[error("ManualTransactionError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ManualTransactionError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ManualTransactionError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ManualTransactionError - ManualTransactionLedgerError: {0}")]
    ManualTransactionLedgerError(#[from] super::ledger::error::ManualTransactionLedgerError),
    #[error("ManualTransactionError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ManualTransactionError - ChartOfAccounts: {0}")]
    ChartOfAccountsError(#[from] chart_of_accounts::error::ChartOfAccountsError),
    #[error("ManualTransactionError - LedgerTransactionInitiatorParseError: {0}")]
    LedgerTransactionInitiatorParseError(#[from] audit::SubjectParseError),
}

es_entity::from_es_entity_error!(ManualTransactionError);

impl ErrorSeverity for ManualTransactionError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::ManualTransactionLedgerError(e) => e.severity(),
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::LedgerTransactionInitiatorParseError(e) => e.severity(),
        }
    }
}
