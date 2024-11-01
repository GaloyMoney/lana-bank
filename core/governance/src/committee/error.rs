use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommitteeError {
    #[error("CommitteeError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CommitteeError - NotFound")]
    NotFound,
    #[error("CommitteeError - MemberAlreadyAdded: {0}")]
    MemberAlreadyAdded(crate::primitives::CommitteeMemberId),
}

es_entity::from_es_entity_error!(CommitteeError);
