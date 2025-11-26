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
    #[error("ManualTransactionError - CalaError: {0}")]
    LedgerError(#[from] cala_ledger::error::LedgerError),
    #[error("ManualTransactionError - CalaAccountSetError: {0}")]
    AccountSetError(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("ManualTransactionError - CalaAccountError: {0}")]
    AccountError(#[from] cala_ledger::account::error::AccountError),
    #[error("ManualTransactionError - CalaTxTemplateError: {0}")]
    TxTemplateError(#[from] cala_ledger::tx_template::error::TxTemplateError),
    #[error("ManualTransactionError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ManualTransactionError - ChartOfAccounts: {0}")]
    ChartOfAccountsError(#[from] chart_of_accounts::error::ChartOfAccountsError),
}

es_entity::from_es_entity_error!(ManualTransactionError);

impl ErrorSeverity for ManualTransactionError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(_) => Level::ERROR,
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::LedgerError(_) => Level::ERROR,
            Self::AccountSetError(_) => Level::ERROR,
            Self::AccountError(_) => Level::ERROR,
            Self::TxTemplateError(_) => Level::ERROR,
            Self::AuthorizationError(_) => Level::ERROR,
            Self::ChartOfAccountsError(_) => Level::ERROR,
        }
    }
}
