use thiserror::Error;

use crate::chart_of_accounts;

#[derive(Error, Debug)]
pub enum AnnualClosingTransactionError {
    #[error("AnnualClosingTransactionError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("AnnualClosingTransactionError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("AnnualClosingTransactionError - CalaError: {0}")]
    CalaError(#[from] cala_ledger::error::LedgerError),
    #[error("AnnualClosingTransactionError - CalaTxTemplateError: {0}")]
    TxTemplateError(#[from] cala_ledger::tx_template::error::TxTemplateError),
    #[error("AnnualClosingTransactionError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("AnnualClosingTransactionError - ChartOfAccounts: {0}")]
    ChartOfAccountsError(#[from] chart_of_accounts::error::ChartOfAccountsError),
    #[error("AnnualClosingTransactionError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(AnnualClosingTransactionError);
