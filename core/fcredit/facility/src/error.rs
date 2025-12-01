use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreCreditFacilityError {
    #[error("CoreCreditFacilityError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreCreditFacilityError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CoreCreditFacilityError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CoreCreditFacilityError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
}

es_entity::from_es_entity_error!(CoreCreditFacilityError);
