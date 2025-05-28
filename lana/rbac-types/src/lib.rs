#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod action;
mod object;

use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};

use core_access::UserId;
use core_customer::CustomerId;

pub use action::*;
pub use object::*;

const SYSTEM_SUBJECT_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
pub const ROLE_NAME_ACCOUNTANT: &str = "accountant";
pub const ROLE_NAME_ADMIN: &str = "admin";
pub const ROLE_NAME_BANK_MANAGER: &str = "bank-manager";

#[derive(Clone, PartialEq, Eq, Copy, async_graphql::Enum)]
pub enum PermissionSetName {
    CustomerWriter,
    CustomerViewer,
}

impl std::str::FromStr for PermissionSetName {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use PermissionSetName::*;
        match s {
            core_customer::PERMISSION_SET_CUSTOMER_WRITER => Ok(CustomerWriter),
            core_customer::PERMISSION_SET_CUSTOMER_VIEWER => Ok(CustomerViewer),
            _ => Err(strum::ParseError::VariantNotFound),
        }
    }
}

#[derive(Clone, Copy, Debug, strum::EnumDiscriminants, Serialize, Deserialize)]
#[strum_discriminants(derive(strum::AsRefStr, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum Subject {
    Customer(CustomerId),
    User(UserId),
    System,
}

impl audit::SystemSubject for Subject {
    fn system() -> Self {
        Subject::System
    }
}

impl std::str::FromStr for Subject {
    type Err = ParseSubjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(ParseSubjectError::InvalidSubjectFormat);
        }

        let id: uuid::Uuid = parts[1].parse()?;
        use SubjectDiscriminants::*;
        let res = match SubjectDiscriminants::from_str(parts[0])? {
            Customer => Subject::Customer(CustomerId::from(id)),
            User => Subject::User(UserId::from(id)),
            System => Subject::System,
        };
        Ok(res)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseSubjectError {
    #[error("ParseSubjectError - Strum: {0}")]
    Strum(#[from] strum::ParseError),
    #[error("ParseSubjectError - Uuid: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("ParseSubjectError - InvalidSubjectFormat")]
    InvalidSubjectFormat,
}

impl From<UserId> for Subject {
    fn from(s: UserId) -> Self {
        Subject::User(s)
    }
}

impl From<CustomerId> for Subject {
    fn from(s: CustomerId) -> Self {
        Subject::Customer(s)
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id: uuid::Uuid = match self {
            Subject::Customer(id) => id.into(),
            Subject::User(id) => id.into(),
            Subject::System => SYSTEM_SUBJECT_ID,
        };
        write!(f, "{}:{}", SubjectDiscriminants::from(self).as_ref(), id)?;
        Ok(())
    }
}

impl TryFrom<&Subject> for deposit::DepositAccountHolderId {
    type Error = &'static str;

    fn try_from(value: &Subject) -> Result<Self, Self::Error> {
        match value {
            Subject::Customer(id) => Ok(deposit::DepositAccountHolderId::from(*id)),
            _ => Err("Subject is not Customer"),
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
