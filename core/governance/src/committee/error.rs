use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CommitteeError {
    #[error("CommitteeError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CommitteeError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CommitteeError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CommitteeError - MemberAlreadyAdded: {0}")]
    MemberAlreadyAdded(crate::primitives::CommitteeMemberId),
}

es_entity::from_es_entity_error!(CommitteeError);

impl ErrorSeverity for CommitteeError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::MemberAlreadyAdded(_) => Level::WARN,
        }
    }
}
