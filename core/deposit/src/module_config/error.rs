use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositConfigError {
    #[error("CommitteeError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CommitteeError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CommitteeError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("TermsError - UninitializedField: {0}")]
    UninitializedField(#[from] derive_builder::UninitializedFieldError),
}

es_entity::from_es_entity_error!(DepositConfigError);
