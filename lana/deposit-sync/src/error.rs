use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum DepositSyncError {
    #[error("DepositSyncError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl ErrorSeverity for DepositSyncError {
    fn severity(&self) -> Level {
        match self {
            Self::RegisterEventHandler(_) => Level::ERROR,
        }
    }
}
