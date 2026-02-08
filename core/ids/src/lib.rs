#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use cala_ledger::primitives::TransactionId as LedgerTransactionId;

pub use audit::SystemActor;

es_entity::entity_id! {
    UserId,
    CustomerId,
    CreditFacilityId,
    DisbursalId,
    PaymentId,
    InterestAccrualCycleId,
    TermsTemplateId,
    ReportId,
    ContractCreationId;

    UserId => governance::CommitteeMemberId,

    CustomerId => document_storage::ReferenceId,
    CustomerId => public_id::PublicIdTargetId,

    CreditFacilityId => governance::ApprovalProcessId,
    DisbursalId => governance::ApprovalProcessId,

    ReportId => job::JobId,
    CreditFacilityId => job::JobId,
    InterestAccrualCycleId => job::JobId,

    DisbursalId => LedgerTransactionId,
    PaymentId => LedgerTransactionId,
}

#[derive(Clone, Debug, strum::EnumDiscriminants, Serialize, Deserialize)]
#[strum_discriminants(derive(strum::AsRefStr, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum Subject {
    Customer(CustomerId),
    User(UserId),
    System(SystemActor),
}

impl audit::SystemSubject for Subject {
    fn system(actor: SystemActor) -> Self {
        Subject::System(actor)
    }
}

impl From<UserId> for Subject {
    fn from(id: UserId) -> Self {
        Subject::User(id)
    }
}

impl From<CustomerId> for Subject {
    fn from(id: CustomerId) -> Self {
        Subject::Customer(id)
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Subject::Customer(id) => {
                let uuid: uuid::Uuid = (*id).into();
                write!(f, "{}:{}", SubjectDiscriminants::from(self).as_ref(), uuid)
            }
            Subject::User(id) => {
                let uuid: uuid::Uuid = (*id).into();
                write!(f, "{}:{}", SubjectDiscriminants::from(self).as_ref(), uuid)
            }
            Subject::System(actor) => {
                write!(f, "{}:{}", SubjectDiscriminants::from(self).as_ref(), actor)
            }
        }
    }
}

impl FromStr for Subject {
    type Err = SubjectParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(SubjectParseError::InvalidSubjectFormat);
        }

        use SubjectDiscriminants::*;
        let res = match SubjectDiscriminants::from_str(parts[0])? {
            Customer => {
                let id: uuid::Uuid = parts[1].parse()?;
                Subject::Customer(CustomerId::from(id))
            }
            User => {
                let id: uuid::Uuid = parts[1].parse()?;
                Subject::User(UserId::from(id))
            }
            System => {
                let actor = SystemActor::from(parts[1].to_string());
                Subject::System(actor)
            }
        };
        Ok(res)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SubjectParseError {
    #[error("SubjectParseError - Strum: {0}")]
    Strum(#[from] strum::ParseError),
    #[error("SubjectParseError - Uuid: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("SubjectParseError - InvalidSubjectFormat")]
    InvalidSubjectFormat,
}

impl tracing_utils::ErrorSeverity for SubjectParseError {
    fn severity(&self) -> tracing::Level {
        match self {
            Self::Strum(_) => tracing::Level::WARN,
            Self::Uuid(_) => tracing::Level::WARN,
            Self::InvalidSubjectFormat => tracing::Level::WARN,
        }
    }
}

impl TryFrom<&Subject> for CustomerId {
    type Error = &'static str;

    fn try_from(value: &Subject) -> Result<Self, Self::Error> {
        match value {
            Subject::Customer(id) => Ok(*id),
            _ => Err("Subject is not Customer"),
        }
    }
}

impl TryFrom<&Subject> for UserId {
    type Error = &'static str;

    fn try_from(value: &Subject) -> Result<Self, Self::Error> {
        match value {
            Subject::User(id) => Ok(*id),
            _ => Err("Subject is not User"),
        }
    }
}

impl TryFrom<&Subject> for governance::CommitteeMemberId {
    type Error = &'static str;

    fn try_from(value: &Subject) -> Result<Self, Self::Error> {
        match value {
            Subject::User(id) => Ok(Self::from(*id)),
            _ => Err("Subject is not User"),
        }
    }
}
