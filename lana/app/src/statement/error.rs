use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatementError {
    #[error("StatementError - ConversionError: {0}")]
    ConversionError(#[from] core_money::ConversionError),
}
