use thiserror::Error;

use crate::chart_of_accounts;

#[derive(Error, Debug)]
pub enum LedgerClosingError {
    #[error("LedgerClosingError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LedgerClosingError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("LedgerClosingError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("LedgerClosingError - CalaError: {0}")]
    LedgerError(#[from] cala_ledger::error::LedgerError),
    #[error("LedgerClosingError - CalaAccountSetError: {0}")]
    AccountSetError(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("LedgerClosingError - CalaAccountError: {0}")]
    AccountError(#[from] cala_ledger::account::error::AccountError),
    #[error("LedgerClosingError - CalaTxTemplateError: {0}")]
    TxTemplateError(#[from] cala_ledger::tx_template::error::TxTemplateError),
    #[error("LedgerClosingError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("LedgerClosingError - ChartOfAccounts: {0}")]
    ChartOfAccountsError(#[from] chart_of_accounts::error::ChartOfAccountsError),
}

es_entity::from_es_entity_error!(LedgerClosingError);
