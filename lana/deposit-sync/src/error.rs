use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositSyncError {
    #[error("DepositSyncError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("DepositSyncError - SumsubError: {0}")]
    Sumsub(#[from] sumsub::SumsubError),
    #[error("DepositSyncError - CoreMoneyError: {0}")]
    CoreMoney(#[from] core_money::ConversionError),
    #[error("DepositSyncError - DecimalConversionError: {0}")]
    DecimalConversion(#[from] rust_decimal::Error),
}
