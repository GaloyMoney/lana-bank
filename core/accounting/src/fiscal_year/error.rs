use crate::primitives::ChartId;
use chrono::NaiveDate;
use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

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
    #[error("FiscalYearError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::DomainConfigError),
    #[error("FiscalYearError - FiscalYearClosingMappingNotConfigured")]
    FiscalYearClosingMappingNotConfigured,
    #[error(
        "FiscalYearError - FiscalYearClosingMappingChartMismatch: config chart is {config_chart_id} but fiscal year chart is {fiscal_year_chart_id}"
    )]
    FiscalYearClosingMappingChartMismatch {
        config_chart_id: ChartId,
        fiscal_year_chart_id: ChartId,
    },
    #[error("FiscalYearError - LastMonthNotClosed")]
    LastMonthNotClosed,
    #[error("FiscalYearError - AllMonthsAlreadyClosed")]
    AllMonthsAlreadyClosed,
    #[error("FiscalYearError - AlreadyOpened")]
    AlreadyOpened,
    #[error("FiscalYearError - FiscalYearNotInitializedForChart: {0}")]
    FiscalYearNotInitializedForChart(ChartId),
    #[error("FiscalYearError - FiscalYearWithInvalidOpenedAsOf: {0}")]
    FiscalYearWithInvalidOpenedAsOf(NaiveDate),
}

es_entity::from_es_entity_error!(FiscalYearError);

impl ErrorSeverity for FiscalYearError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::DomainConfigError(e) => e.severity(),
            Self::FiscalYearClosingMappingNotConfigured => Level::ERROR,
            Self::FiscalYearClosingMappingChartMismatch { .. } => Level::ERROR,
            Self::LastMonthNotClosed => Level::ERROR,
            Self::AllMonthsAlreadyClosed => Level::ERROR,
            Self::AlreadyOpened => Level::ERROR,
            Self::FiscalYearNotInitializedForChart(_) => Level::ERROR,
            Self::FiscalYearWithInvalidOpenedAsOf(_) => Level::ERROR,
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
