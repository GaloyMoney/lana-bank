use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChartOfAccountsError {
    #[error("ChartOfAccountsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ChartOfAccountsError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ChartOfAccountsError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ChartOfAccountsError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ChartOfAccountsError - CodeNotFoundInChart: {0}")]
    CodeNotFoundInChart(crate::primitives::AccountCode),
    #[error("ChartOfAccountsError - CsvParseError: {0}")]
    CsvParse(#[from] super::CsvParseError),
    #[error("ChartOfAccountsError - CalaLedgerError: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("ChartOfAccountsError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("ChartOfAccountsError - CalaAccountSetError: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
}

es_entity::from_es_entity_error!(ChartOfAccountsError);
