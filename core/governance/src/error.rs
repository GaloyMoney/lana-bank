use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("GovernanceError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("GovernanceError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("GovernanceError - CommitteeError: {0}")]
    CommitteeError(#[from] crate::committee::error::CommitteeError),
    #[error("GovernanceError - PolicyError: {0}")]
    PolicyError(#[from] crate::policy::error::PolicyError),
    #[error("GovernanceError - ApprovalProcessError: {0}")]
    ApprovalProcessError(#[from] crate::approval_process::error::ApprovalProcessError),
    #[error("GovernanceError - Audit: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("GovernanceError - SubjectIsNotCommitteeMember")]
    SubjectIsNotCommitteeMember,
}

impl ErrorSeverity for GovernanceError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::CommitteeError(e) => e.severity(),
            Self::PolicyError(e) => e.severity(),
            Self::ApprovalProcessError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::SubjectIsNotCommitteeMember => Level::WARN,
        }
    }
}
