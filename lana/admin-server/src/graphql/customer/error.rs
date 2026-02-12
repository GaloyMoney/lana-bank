use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum CustomerError {
    #[error("CustomerError - MissingValueForFilterField: {0}")]
    MissingValueForFilterField(String),
}
