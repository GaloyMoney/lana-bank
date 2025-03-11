use thiserror::Error;

use super::AccountCode;

#[derive(Error, Debug)]
pub enum AltChartError {
    #[error("AltChartError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("AltChartError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("AltChartError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CreditFacilityError - CodeNotFoundInChart: {0}")]
    CodeNotFoundInChart(AccountCode),
}

es_entity::from_es_entity_error!(AltChartError);
