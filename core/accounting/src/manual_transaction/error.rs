use thiserror::Error;

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
    #[error("ManualTransactionError - Unknown account code: {0}")]
    UnknownAccountCode(String),
}

es_entity::from_es_entity_error!(ManualTransactionError);
