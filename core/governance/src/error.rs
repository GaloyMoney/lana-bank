use thiserror::Error;

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
