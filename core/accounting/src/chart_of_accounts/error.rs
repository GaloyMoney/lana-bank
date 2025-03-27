use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChartError {
    #[error("ChartError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ChartError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ChartError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ChartError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ChartError - CodeNotFoundInChart: {0}")]
    CodeNotFoundInChart(crate::primitives::AccountCode),
    #[error("ChartError - CsvParseError: {0}")]
    CsvParse(#[from] super::CsvParseError),
    #[error("ChartError - CalaLedgerError: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("ChartError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("ChartError - CalaAccountSetError: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
}

es_entity::from_es_entity_error!(ChartError);
