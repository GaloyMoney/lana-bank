use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum DepositSyncError {
    #[error("DepositSyncError - JobError: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("DepositSyncError - SumsubError: {0}")]
    Sumsub(#[from] sumsub::SumsubError),
    #[error("DepositSyncError - CoreMoneyError: {0}")]
    CoreMoney(#[from] money::ConversionError),
    #[error("DepositSyncError - DecimalConversionError: {0}")]
    DecimalConversion(#[from] rust_decimal::Error),
    #[error("DepositSyncError - CoreDepositError: {0}")]
    CoreDeposit(#[from] core_deposit::error::CoreDepositError),
    #[error("DepositSyncError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl ErrorSeverity for DepositSyncError {
    fn severity(&self) -> Level {
        match self {
            Self::Job(_) => Level::ERROR,
            Self::Sumsub(e) => e.severity(),
            Self::CoreMoney(e) => e.severity(),
            Self::DecimalConversion(_) => Level::ERROR,
            Self::CoreDeposit(e) => e.severity(),
            Self::RegisterEventHandler(_) => Level::ERROR,
        }
    }
}
