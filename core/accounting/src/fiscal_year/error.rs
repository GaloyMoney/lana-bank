use crate::primitives::ChartId;
use chrono::NaiveDate;
use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum FiscalYearError {
    #[error("FiscalYearError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("FiscalYearError - ParseIntError: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("FiscalYearError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("FiscalYearError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("FiscalYearError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("FiscalYearError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] crate::chart_of_accounts::error::ChartOfAccountsError),
    #[error("FiscalYearError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::error::DomainConfigError),
    #[error("FiscalYearError - LastMonthNotClosed")]
    LastMonthNotClosed,
    #[error("FiscalYearError - MonthHasNotEnded")]
    MonthHasNotEnded,
    #[error("FiscalYearError - AllMonthsAlreadyClosed")]
    AllMonthsAlreadyClosed,
    #[error("FiscalYearError - AlreadyOpened")]
    AlreadyOpened,
    #[error("FiscalYearError - FiscalYearNotInitializedForChart: {0}")]
    FiscalYearNotInitializedForChart(ChartId),
    #[error("FiscalYearError - FiscalYearWithInvalidOpenedAsOf: {0}")]
    FiscalYearWithInvalidOpenedAsOf(NaiveDate),
    #[error("FiscalYearError - InvalidYearString: {0}")]
    InvalidYearString(String),
    #[error("FiscalYearError - FiscalYearConfigAlreadyExists")]
    FiscalYearConfigAlreadyExists,
}

es_entity::from_es_entity_error!(FiscalYearError);

impl ErrorSeverity for FiscalYearError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::ParseIntError(_) => Level::WARN,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::DomainConfigError(e) => e.severity(),
            Self::LastMonthNotClosed => Level::ERROR,
            Self::AllMonthsAlreadyClosed => Level::ERROR,
            Self::AlreadyOpened => Level::ERROR,
            Self::FiscalYearNotInitializedForChart(_) => Level::ERROR,
            Self::FiscalYearWithInvalidOpenedAsOf(_) => Level::ERROR,
            Self::InvalidYearString(_) => Level::WARN,
            Self::MonthHasNotEnded => Level::ERROR,
            Self::FiscalYearConfigAlreadyExists => Level::ERROR,
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
