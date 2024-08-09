use thiserror::Error;

use crate::primitives::CustomerId;

#[derive(Error, Debug)]
pub enum CustomerError {
    #[error("CustomerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustomerError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("CustomerError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
    #[error("CustomerError - CouldNotFindById: {0}")]
    CouldNotFindById(CustomerId),
    #[error("CustomerError - UnexpectedCurrency")]
    UnexpectedCurrency,
    #[error("CustomerError - KratosClientError: {0}")]
    KratosClientError(#[from] super::kratos::error::KratosClientError),
    #[error("CustomerError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("CustomerError - CouldNotFindByEmail: {0}")]
    CouldNotFindByEmail(String),
    #[error("CustomerError - AuditError: ${0}")]
    AuditError(#[from] crate::audit::error::AuditError),
}
