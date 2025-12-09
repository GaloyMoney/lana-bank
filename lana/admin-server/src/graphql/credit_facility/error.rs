use thiserror::Error;
use tracing::Level;

use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CreditFacilityError {
    #[error("CreditFacilityError - MissingValueForFilterField: {0}")]
    MissingValueForFilterField(String),
}

impl ErrorSeverity for CreditFacilityError {
    fn severity(&self) -> Level {
        match self {
            Self::MissingValueForFilterField(_) => Level::ERROR,
        }
    }
}
