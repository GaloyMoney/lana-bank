use thiserror::Error;

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
}
