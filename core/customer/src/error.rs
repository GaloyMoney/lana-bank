use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::{
    party,
    party::repo::{PartyCreateError, PartyFindError, PartyModifyError, PartyQueryError},
    prospect,
    prospect::repo::{
        ProspectCreateError, ProspectFindError, ProspectModifyError, ProspectQueryError,
    },
    repo::{CustomerCreateError, CustomerFindError, CustomerModifyError, CustomerQueryError},
};

#[derive(Error, Debug)]
pub enum CustomerError {
    #[error("CustomerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustomerError - Create: {0}")]
    Create(#[from] CustomerCreateError),
    #[error("CustomerError - Modify: {0}")]
    Modify(#[from] CustomerModifyError),
    #[error("CustomerError - Find: {0}")]
    Find(#[from] CustomerFindError),
    #[error("CustomerError - Query: {0}")]
    Query(#[from] CustomerQueryError),
    #[error("CustomerError - UnexpectedCurrency")]
    UnexpectedCurrency,
    #[error("CustomerError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CustomerError - AuditError: ${0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CustomerError - SubjectIsNotCustomer")]
    SubjectIsNotCustomer,
    #[error("CustomerError - DocumentStorageError: {0}")]
    DocumentStorageError(#[from] document_storage::error::DocumentStorageError),
    #[error("CustomerError - PublicIdError: {0}")]
    PublicIdError(#[from] public_id::PublicIdError),
    #[error("CustomerError - ProspectError: {0}")]
    ProspectError(#[from] prospect::ProspectError),
    #[error("CustomerError - PartyError: {0}")]
    PartyError(#[from] party::PartyError),
    #[error("CustomerError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::DomainConfigError),
    #[error("CustomerError - CustomerIsClosed")]
    CustomerIsClosed,
    #[error("CustomerError - CustomerNotEligibleForProduct")]
    CustomerNotEligibleForProduct,
    #[error("CustomerError - ManualConversionNotAllowed")]
    ManualConversionNotAllowed,
}

impl From<ProspectCreateError> for CustomerError {
    fn from(e: ProspectCreateError) -> Self {
        Self::ProspectError(e.into())
    }
}

impl From<ProspectFindError> for CustomerError {
    fn from(e: ProspectFindError) -> Self {
        Self::ProspectError(e.into())
    }
}

impl From<ProspectModifyError> for CustomerError {
    fn from(e: ProspectModifyError) -> Self {
        Self::ProspectError(e.into())
    }
}

impl From<ProspectQueryError> for CustomerError {
    fn from(e: ProspectQueryError) -> Self {
        Self::ProspectError(e.into())
    }
}

impl From<PartyCreateError> for CustomerError {
    fn from(e: PartyCreateError) -> Self {
        Self::PartyError(e.into())
    }
}

impl From<PartyFindError> for CustomerError {
    fn from(e: PartyFindError) -> Self {
        Self::PartyError(e.into())
    }
}

impl From<PartyModifyError> for CustomerError {
    fn from(e: PartyModifyError) -> Self {
        Self::PartyError(e.into())
    }
}

impl From<PartyQueryError> for CustomerError {
    fn from(e: PartyQueryError) -> Self {
        Self::PartyError(e.into())
    }
}

impl ErrorSeverity for CustomerError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::UnexpectedCurrency => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::SubjectIsNotCustomer => Level::WARN,
            Self::DocumentStorageError(e) => e.severity(),
            Self::PublicIdError(e) => e.severity(),
            Self::ProspectError(e) => e.severity(),
            Self::PartyError(e) => e.severity(),
            Self::DomainConfigError(_) => Level::ERROR,
            Self::CustomerIsClosed => Level::WARN,
            Self::CustomerNotEligibleForProduct => Level::WARN,
            Self::ManualConversionNotAllowed => Level::WARN,
        }
    }
}
