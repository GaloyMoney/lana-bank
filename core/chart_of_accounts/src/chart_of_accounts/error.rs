use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChartOfAccountError {
    #[error("ChartOfAccountError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ChartOfAccountError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ChartOfAccountError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ChartOfAccountError - InvalidChartOfAccountCodeStr")]
    InvalidChartOfAccountCodeStr,
}

es_entity::from_es_entity_error!(ChartOfAccountError);
