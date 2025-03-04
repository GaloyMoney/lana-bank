use thiserror::Error;

use crate::email::error::EmailError;

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("NotificationError - Email: {0}")]
    Email(#[from] EmailError),
}
