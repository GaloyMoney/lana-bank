use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositSyncError {
    #[error("DepositSyncError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("DepositSyncError - SumsubError: {0}")]
    Sumsub(#[from] sumsub::SumsubError),
}
