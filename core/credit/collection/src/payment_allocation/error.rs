use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    PaymentAllocationCreateError, PaymentAllocationFindError, PaymentAllocationModifyError,
    PaymentAllocationQueryError,
};

#[derive(Error, Debug)]
pub enum PaymentAllocationError {
    #[error("PaymentAllocationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PaymentAllocationError - Create: {0}")]
    Create(#[from] PaymentAllocationCreateError),
    #[error("PaymentAllocationError - Modify: {0}")]
    Modify(#[from] PaymentAllocationModifyError),
    #[error("PaymentAllocationError - Find: {0}")]
    Find(#[from] PaymentAllocationFindError),
    #[error("PaymentAllocationError - Query: {0}")]
    Query(#[from] PaymentAllocationQueryError),
    #[error("PaymentAllocationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

impl ErrorSeverity for PaymentAllocationError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
        }
    }
}
