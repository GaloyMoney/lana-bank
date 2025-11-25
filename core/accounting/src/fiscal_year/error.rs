use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

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
    #[error("FiscalYearError - NotReadyForAnnualClose")]
    NotReadyForAnnualClose,
    #[error("FiscalYearError - AlreadyClosed")]
    AlreadyClosed,
}

es_entity::from_es_entity_error!(FiscalYearError);

impl ErrorSeverity for FiscalYearError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(_) => Level::ERROR,
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
        }
    }
}
