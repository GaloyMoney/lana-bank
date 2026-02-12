use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum CreditFacilityError {
    #[error("CreditFacilityError - MissingValueForFilterField: {0}")]
    MissingValueForFilterField(String),
}
