use thiserror::Error;

use crate::primitives::*;

#[derive(Error, Debug)]
pub enum FundingLinkError {
    #[error("FundingLinkError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("FundingLinkError - EsEntity: {0}")]
    EsEntity(#[from] es_entity::EsEntityError),
    #[error("FundingLinkError - CursorDestructure: {0}")]
    CursorDestructure(#[from] es_entity::CursorDestructureError),
    #[error("FundingLinkError - FundingLinkNotFound: {0}")]
    NotFound(FundingLinkId),
    #[error("FundingLinkError - LinkAlreadyBroken")]
    LinkAlreadyBroken,
    #[error("FundingLinkError - LinkNotActive")]
    LinkNotActive,
}

