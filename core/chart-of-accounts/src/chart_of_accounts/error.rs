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
    #[error("ChartError - NodeAlreadyRegisteredByReference: '{0}'")]
    NodeAlreadyRegisteredByReference(String),
    #[error("ChartError - NodeAlreadyRegisteredByPath: '{0}'")]
    NodeAlreadyRegisteredByPath(String),
    #[error("ChartError - InvalidCodeLength: Account code length {0} does not match any valid lengths: {1:?}")]
    InvalidCodeLength(usize, Vec<usize>),
    #[error("ChartError - NonDigitCharactersInCode: '{0}'")]
    NonDigitCharactersInCode(String),
}

es_entity::from_es_entity_error!(ChartError);
