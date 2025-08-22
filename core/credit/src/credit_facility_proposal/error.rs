use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreditFacilityProposalError {
    #[error("CreditFacilityProposalError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityProposalError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CreditFacilityProposalError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CreditFacilityProposalError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
}

es_entity::from_es_entity_error!(CreditFacilityProposalError);
