use thiserror::Error;
use tracing::Level;

use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CustomerError {
    #[error("CustomerError - MissingValueForFilterField: {0}")]
    MissingValueForFilterField(String),
}

impl ErrorSeverity for CustomerError {
    fn severity(&self) -> Level {
        match self {
            Self::MissingValueForFilterField(_) => Level::ERROR,
        }
    }
}
