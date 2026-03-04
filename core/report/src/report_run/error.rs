use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    ReportRunCreateError, ReportRunFindError, ReportRunModifyError, ReportRunQueryError,
};

#[derive(Error, Debug)]
pub enum ReportRunError {
    #[error("ReportRunError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ReportRunError - Create: {0}")]
    Create(#[from] ReportRunCreateError),
    #[error("ReportRunError - Modify: {0}")]
    Modify(#[from] ReportRunModifyError),
    #[error("ReportRunError - Find: {0}")]
    Find(#[from] ReportRunFindError),
    #[error("ReportRunError - Query: {0}")]
    Query(#[from] ReportRunQueryError),
}

impl ErrorSeverity for ReportRunError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
        }
    }
}
