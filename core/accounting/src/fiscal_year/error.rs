use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::primitives::ChartId;

#[derive(Error, Debug)]
pub enum FiscalYearError {
    #[error("FiscalYearError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("FiscalYearError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("FiscalYearError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("FiscalYearError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("FiscalYearError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] crate::chart_of_accounts::error::ChartOfAccountsError),
    #[error("FiscalYearError - LastMonthNotClosed")]
    LastMonthNotClosed,
    #[error("FiscalYearError - AlreadyClosed")]
    AlreadyClosed,
    #[error("FiscalYearError - AlreadyOpened")]
    AlreadyOpened,
    #[error("FiscalYearError - FiscalYearNotInitializedForChart: {0}")]
    FiscalYearNotInitializedForChart(ChartId),
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
            Self::AllMonthsNotClosed => Level::ERROR,
            Self::AlreadyClosed => Level::ERROR,
            Self::YearAlreadyOpened => Level::ERROR,
            Self::FiscalYearNotInitializedForChart(_) => Level::ERROR,
        }
    }
}
impl From<sqlx::Error> for FiscalYearError {
    fn from(error: sqlx::Error) -> Self {
        if let Some(err) = error.as_database_error()
            && let Some(constraint) = err.constraint()
            && constraint.contains("reference")
        {
            return Self::AlreadyOpened;
        }
        Self::Sqlx(error)
    }
}
