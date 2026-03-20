use thiserror::Error;

use crate::primitives::EodProcessStatus;

pub use super::repo::{
    EodProcessCreateError, EodProcessFindError, EodProcessModifyError, EodProcessQueryError,
};

#[derive(Error, Debug)]
pub enum EodProcessError {
    #[error("EodProcessError - Create: {0}")]
    Create(#[from] EodProcessCreateError),
    #[error("EodProcessError - Modify: {0}")]
    Modify(#[from] EodProcessModifyError),
    #[error("EodProcessError - Find: {0}")]
    Find(#[from] EodProcessFindError),
    #[error("EodProcessError - Query: {0}")]
    Query(#[from] EodProcessQueryError),
    #[error("Invalid state transition: cannot {attempted} in state {current}")]
    InvalidStateTransition {
        current: EodProcessStatus,
        attempted: &'static str,
    },
    #[error("EodProcessError - MissingJobIds: expected job IDs not found in event stream")]
    MissingJobIds,
}
