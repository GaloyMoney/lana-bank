use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug, strum::IntoStaticStr)]
pub enum CreditFacilityHistoryError {
    #[error("CreditFacilityHistoryError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityHistoryError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CreditFacilityHistoryError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("CoreCreditError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
}

impl ErrorSeverity for CreditFacilityHistoryError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::RegisterEventHandler(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
        }
    }

    fn variant_name(&self) -> &'static str {
        self.into()
    }
}
