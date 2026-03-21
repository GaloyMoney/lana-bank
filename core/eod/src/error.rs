use thiserror::Error;

use tracing_utils::{ErrorSeverity, Level};

#[derive(Error, Debug)]
pub enum CoreEodError {
    #[error("CoreEodError - EodProcessError: {0}")]
    EodProcessError(#[from] crate::eod_process::error::EodProcessError),
    #[error("CoreEodError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CoreEodError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("CoreEodError - DuplicatePhase: phase '{0}' is registered more than once")]
    DuplicatePhase(String),
}

impl ErrorSeverity for CoreEodError {
    fn severity(&self) -> Level {
        match self {
            Self::EodProcessError(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::RegisterEventHandler(_) => Level::ERROR,
            Self::DuplicatePhase(_) => Level::ERROR,
        }
    }
}
