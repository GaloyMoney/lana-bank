use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountingCsvError {
    #[error("AccountingCsvError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("AccountingCsvError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("AccountingCsvError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("AccountingCsvError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("AccountingCsvError - LedgerAccountError: {0}")]
    LedgerAccountError(#[from] crate::ledger_account::error::LedgerAccountError),
    #[error("AccountingCsvError - StorageError: {0}")]
    StorageError(#[from] cloud_storage::error::StorageError),
    #[error("AccountingCsvError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("AccountingCsvError - DocumentStorageError: {0}")]
    DocumentStorageError(#[from] document_storage::error::DocumentStorageError),
    #[error("AccountingCsvError - CsvError: {0}")]
    CsvError(String),
    #[error("AccountingCsvError - UnsupportedCsvType")]
    UnsupportedCsvType,
    #[error("AccountingCsvError - CsvNotReady")]
    CsvNotReady,
    #[error("AccountingCsvError - CsvFileNotFound")]
    CsvFileNotFound,
    #[error("AccountingCsvError - MissingRequiredField: {0}")]
    MissingRequiredField(String),
}

es_entity::from_es_entity_error!(AccountingCsvError);
