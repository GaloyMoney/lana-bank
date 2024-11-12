use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerError {
    #[error("CustomerError - MissingValueForFilterField: {0}")]
    MissingValueForFilterField(String),
}
