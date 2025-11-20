use thiserror::Error;

#[derive(Error, Debug)]
pub enum FiscalYearError {
    #[error("FiscalYearError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("FiscalYearError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("FiscalYearError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("FiscalYearError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("FiscalYearError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] crate::chart_of_accounts::error::ChartOfAccountsError),
}

es_entity::from_es_entity_error!(FiscalYearError);
