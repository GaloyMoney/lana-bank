use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreditFacilityRepaymentPlanError {
    #[error("CreditFacilityRepaymentPlanError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}
