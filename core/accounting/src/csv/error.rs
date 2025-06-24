use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountingCsvExportError {
    #[error("AccountingCsvExportError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("AccountingCsvExportError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("AccountingCsvExportError - LedgerAccountError: {0}")]
    LedgerAccountError(#[from] crate::ledger_account::error::LedgerAccountError),
    #[error("AccountingCsvExportError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("AccountingCsvExportError - DocumentStorageError: {0}")]
    DocumentStorageError(#[from] document_storage::error::DocumentStorageError),
    #[error("AccountingCsvExportError - CsvError: {0}")]
    CsvError(String),
}
