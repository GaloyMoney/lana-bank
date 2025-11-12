use crate::primitives::ChartId;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FiscalYearError {
    #[error("FiscalYearError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("FiscalYearError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("FiscalYearError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("FiscalYearError - PeriodAlreadyClosed")]
    PeriodAlreadyClosed,
    #[error("FiscalYearError - ClosingMetadataNotFound")]
    ClosingMetadataNotFound { chart_id: ChartId },
    #[error("FiscalYearError - ChartIdMismatch")]
    ChartIdMismatch,
    #[error("FiscalYearError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("FiscalYearError - CurrentYearNotFound")]
    CurrentYearNotFound,
    #[error("FiscalYearError - FiscalYearAlreadyInitialized")]
    FiscalYearAlreadyInitialized,
    #[error("FiscalYearError - LedgerError: {0}")]
    Ledger(#[from] super::ledger::error::FiscalYearLedgerError),
    #[error("FiscalYearError - FiscalYearMonthAlreadyClosed")]
    FiscalYearMonthAlreadyClosed,
    #[error("FiscalYearError - CurrentYearNotFoundByChartReference: {0}")]
    CurrentYearNotFoundByChartReference(String),
}

es_entity::from_es_entity_error!(FiscalYearError);
