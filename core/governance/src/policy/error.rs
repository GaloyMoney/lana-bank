use thiserror::Error;

#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("PolicyError - Sqlx: {0}")]
    Sqlx(sqlx::Error),
    #[error("PolicyError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("PolicyError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("PolicyError - DuplicateApprovalProcessType")]
    DuplicateApprovalProcessType,
    #[error("PolicyError - Threshold {1} too high for committee {0}")]
    PolicyThresholdTooHigh(crate::primitives::CommitteeId, usize),
    #[error("PolicyError - Threshold {1} too low for committee {0}")]
    PolicyThresholdTooLow(crate::primitives::CommitteeId, usize),
}

es_entity::from_es_entity_error!(PolicyError);

impl From<sqlx::Error> for PolicyError {
    fn from(error: sqlx::Error) -> Self {
        if let Some(err) = error.as_database_error() {
            if let Some(constraint) = err.constraint() {
                if constraint.contains("type") {
                    return Self::DuplicateApprovalProcessType;
                }
            }
        }
        Self::Sqlx(error)
    }
}
