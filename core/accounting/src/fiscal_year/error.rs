use crate::primitives::ChartId;
use chrono::NaiveDate;
use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    FiscalYearCreateError, FiscalYearFindError, FiscalYearModifyError, FiscalYearQueryError,
};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum FiscalYearError {
    #[error("FiscalYearError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("FiscalYearError - ParseIntError: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("FiscalYearError - Create: {0}")]
    Create(FiscalYearCreateError),
    #[error("FiscalYearError - Modify: {0}")]
    Modify(#[from] FiscalYearModifyError),
    #[error("FiscalYearError - Find: {0}")]
    Find(#[from] FiscalYearFindError),
    #[error("FiscalYearError - Query: {0}")]
    Query(#[from] FiscalYearQueryError),
    #[error("FiscalYearError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("FiscalYearError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] crate::chart_of_accounts::error::ChartOfAccountsError),
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
}

impl ErrorSeverity for FiscalYearError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::ParseIntError(_) => Level::WARN,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::LastMonthNotClosed => Level::WARN,
            Self::AllMonthsAlreadyClosed => Level::ERROR,
            Self::AlreadyOpened => Level::ERROR,
            Self::FiscalYearNotInitializedForChart(_) => Level::ERROR,
            Self::FiscalYearWithInvalidOpenedAsOf(_) => Level::ERROR,
            Self::InvalidYearString(_) => Level::WARN,
            Self::MonthHasNotEnded => Level::WARN,
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

impl From<FiscalYearCreateError> for FiscalYearError {
    fn from(error: FiscalYearCreateError) -> Self {
        if error.was_duplicate() {
            return Self::AlreadyOpened;
        }
        Self::Create(error)
    }
}
