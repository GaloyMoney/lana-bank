use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreditFacilityError {
    #[error("CreditFacilityError - MissingValueForFilterField: {0}")]
    MissingValueForFilterField(String),
}
