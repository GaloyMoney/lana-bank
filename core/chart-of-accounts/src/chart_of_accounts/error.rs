use thiserror::Error;

use crate::AccountCode;

#[derive(Error, Debug)]
pub enum ChartError {
    #[error("ChartError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ChartError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ChartError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ChartError - CodeNotFoundInChart: {0}")]
    CodeNotFoundInChart(AccountCode),
}

es_entity::from_es_entity_error!(ChartError);
