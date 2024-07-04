use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("ConversionError - TryFromIntError: {0}")]
    TryFromIntError(#[from] std::num::TryFromIntError),
}
