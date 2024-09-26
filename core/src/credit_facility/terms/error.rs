use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreditFacilityTermsError {
    #[error("CreditFacilityTermsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityTermsError - TermsNotSet")]
    TermsNotSet,
}
