use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaymentLinkError {
    #[error("PaymentLinkError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PaymentLinkError - EsEntity: {0}")]
    EsEntity(#[from] es_entity::EsEntityError),
    #[error("PaymentLinkError - FundingLink: {0}")]
    FundingLink(#[from] crate::funding_link::error::FundingLinkError),
}

