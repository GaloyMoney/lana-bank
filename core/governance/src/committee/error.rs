use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use super::repo::{
    CommitteeCreateError, CommitteeFindError, CommitteeModifyError, CommitteeQueryError,
};

#[derive(Error, Debug)]
pub enum CommitteeError {
    #[error("CommitteeError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CommitteeError - Create: {0}")]
    Create(#[from] CommitteeCreateError),
    #[error("CommitteeError - Modify: {0}")]
    Modify(#[from] CommitteeModifyError),
    #[error("CommitteeError - Find: {0}")]
    Find(#[from] CommitteeFindError),
    #[error("CommitteeError - Query: {0}")]
    Query(#[from] CommitteeQueryError),
    #[error("CommitteeError - CannotRemoveLastMember")]
    CannotRemoveLastMember,
    #[error("CommitteeError - UninitializedField: {0}")]
    UninitializedField(#[from] derive_builder::UninitializedFieldError),
    #[error("CommitteeError - CommitteeMustHaveAtLeastOneMember")]
    CommitteeMustHaveAtLeastOneMember,
}

impl ErrorSeverity for CommitteeError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::UninitializedField(_) => Level::ERROR,
            Self::CannotRemoveLastMember => Level::WARN,
            Self::CommitteeMustHaveAtLeastOneMember => Level::WARN,
        }
    }
}
