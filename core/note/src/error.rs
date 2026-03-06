use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{NoteCreateError, NoteFindError, NoteModifyError, NoteQueryError};

#[derive(Error, Debug)]
pub enum NoteError {
    #[error("NoteError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("NoteError - Create: {0}")]
    Create(#[from] NoteCreateError),
    #[error("NoteError - Modify: {0}")]
    Modify(#[from] NoteModifyError),
    #[error("NoteError - Find: {0}")]
    Find(#[from] NoteFindError),
    #[error("NoteError - Query: {0}")]
    Query(#[from] NoteQueryError),
}

impl ErrorSeverity for NoteError {
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
