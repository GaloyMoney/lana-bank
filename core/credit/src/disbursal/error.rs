use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum DisbursalError {
    #[error("DisbursalError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DisbursalError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DisbursalError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("DisbursalError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("DisbursalError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("DisbursalError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("DisbursalError - ObligationError: {0}")]
    ObligationError(#[from] core_credit_collection::obligation::error::ObligationError),
    #[error("CreditFacilityError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
}

impl ErrorSeverity for DisbursalError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::GovernanceError(e) => e.severity(),
            Self::ObligationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
        }
    }
}

es_entity::from_es_entity_error!(DisbursalError);
