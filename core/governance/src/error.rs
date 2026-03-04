use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::approval_process::error::{
    ApprovalProcessCreateError, ApprovalProcessFindError, ApprovalProcessModifyError,
    ApprovalProcessQueryError,
};
use crate::committee::error::{
    CommitteeCreateError, CommitteeFindError, CommitteeModifyError, CommitteeQueryError,
};
use crate::policy::error::{
    PolicyCreateError, PolicyFindError, PolicyModifyError, PolicyQueryError,
};

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

// Policy repo error -> GovernanceError (two-hop: repo error -> PolicyError -> GovernanceError)
impl From<PolicyCreateError> for GovernanceError {
    fn from(e: PolicyCreateError) -> Self {
        Self::PolicyError(e.into())
    }
}

impl From<PolicyFindError> for GovernanceError {
    fn from(e: PolicyFindError) -> Self {
        Self::PolicyError(e.into())
    }
}

impl From<PolicyModifyError> for GovernanceError {
    fn from(e: PolicyModifyError) -> Self {
        Self::PolicyError(e.into())
    }
}

impl From<PolicyQueryError> for GovernanceError {
    fn from(e: PolicyQueryError) -> Self {
        Self::PolicyError(e.into())
    }
}

// Committee repo error -> GovernanceError (two-hop: repo error -> CommitteeError -> GovernanceError)
impl From<CommitteeCreateError> for GovernanceError {
    fn from(e: CommitteeCreateError) -> Self {
        Self::CommitteeError(e.into())
    }
}

impl From<CommitteeFindError> for GovernanceError {
    fn from(e: CommitteeFindError) -> Self {
        Self::CommitteeError(e.into())
    }
}

impl From<CommitteeModifyError> for GovernanceError {
    fn from(e: CommitteeModifyError) -> Self {
        Self::CommitteeError(e.into())
    }
}

impl From<CommitteeQueryError> for GovernanceError {
    fn from(e: CommitteeQueryError) -> Self {
        Self::CommitteeError(e.into())
    }
}

// ApprovalProcess repo error -> GovernanceError (two-hop: repo error -> ApprovalProcessError -> GovernanceError)
impl From<ApprovalProcessCreateError> for GovernanceError {
    fn from(e: ApprovalProcessCreateError) -> Self {
        Self::ApprovalProcessError(e.into())
    }
}

impl From<ApprovalProcessFindError> for GovernanceError {
    fn from(e: ApprovalProcessFindError) -> Self {
        Self::ApprovalProcessError(e.into())
    }
}

impl From<ApprovalProcessModifyError> for GovernanceError {
    fn from(e: ApprovalProcessModifyError) -> Self {
        Self::ApprovalProcessError(e.into())
    }
}

impl From<ApprovalProcessQueryError> for GovernanceError {
    fn from(e: ApprovalProcessQueryError) -> Self {
        Self::ApprovalProcessError(e.into())
    }
}
