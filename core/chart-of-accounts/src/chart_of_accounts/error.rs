use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChartOfAccountError {
    #[error("ChartOfAccountError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ChartOfAccountError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ChartOfAccountError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ChartOfAccountError - ChartOfAccountCodeError: '{0}'")]
    ChartOfAccountCodeError(#[from] crate::code::error::ChartOfAccountCodeError),
    #[error("ChartOfAccountError - ControlAccountAlreadyRegistered: '{0}'")]
    ControlAccountAlreadyRegistered(String),
    #[error("ChartOfAccountError - ControlSubAccountAlreadyRegistered: '{0}'")]
    ControlSubAccountAlreadyRegistered(String),
}

es_entity::from_es_entity_error!(ChartOfAccountError);
