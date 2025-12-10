use chrono::ParseError;
use chrono_tz::ParseError as TzParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeEventsError {
    #[error("TimeEventsError")]
    GeneralError,
    #[error("TzParseError: {0}")]
    TzParseError(#[from] TzParseError),
    #[error("ParseError: {0}")]
    ParseError(#[from] ParseError),
}
