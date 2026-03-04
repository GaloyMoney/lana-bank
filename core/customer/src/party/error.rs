use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{PartyCreateError, PartyFindError, PartyModifyError, PartyQueryError};

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum PartyError {
    #[error("PartyError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PartyError - Create: {0}")]
    Create(#[from] PartyCreateError),
    #[error("PartyError - Modify: {0}")]
    Modify(#[from] PartyModifyError),
    #[error("PartyError - Find: {0}")]
    Find(#[from] PartyFindError),
    #[error("PartyError - Query: {0}")]
    Query(#[from] PartyQueryError),
    #[error("PartyError - EmailAlreadyExists")]
    EmailAlreadyExists,
    #[error("PartyError - TelegramHandleAlreadyExists")]
    TelegramHandleAlreadyExists,
}

impl PartyError {
    pub fn from_db_error(error: PartyError) -> PartyError {
        match &error {
            PartyError::Sqlx(sqlx::Error::Database(db_err)) => match db_err.constraint() {
                Some("core_parties_email_key") => PartyError::EmailAlreadyExists,
                Some("core_parties_telegram_handle_key") => PartyError::TelegramHandleAlreadyExists,
                _ => error,
            },
            _ => error,
        }
    }
}

impl ErrorSeverity for PartyError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::EmailAlreadyExists => Level::WARN,
            Self::TelegramHandleAlreadyExists => Level::WARN,
        }
    }
}
