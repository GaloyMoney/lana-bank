use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChartError {
    #[error("ChartError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ChartError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ChartError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ChartError - ChartPathError: '{0}'")]
    ChartPathError(#[from] crate::path::error::ChartPathError),
    #[error("ChartError - ControlAccountAlreadyRegistered: '{0}'")]
    ControlAccountAlreadyRegistered(String),
    #[error("ChartError - ControlSubAccountAlreadyRegistered: '{0}'")]
    ControlSubAccountAlreadyRegistered(String),
    #[error("ChartError - CodeDoesNotExistInChart: '{0}'")]
    CodeDoesNotExistInChart(String),
}

es_entity::from_es_entity_error!(ChartError);
