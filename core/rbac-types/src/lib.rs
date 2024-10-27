#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod action;
mod object;

use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};

use shared_primitives::{CustomerId, UserId};

pub use action::*;
pub use object::*;

#[derive(
    async_graphql::Enum,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::EnumString,
    strum::Display,
)]
#[strum(serialize_all = "kebab-case")]
pub enum Role {
    Superuser,
    Admin,
    BankManager,
    Accountant,
}

impl std::fmt::Display for SystemNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemNode::Init => SYSTEM_INIT.fmt(f),
            SystemNode::Core => SYSTEM_CORE.fmt(f),
            SystemNode::Kratos => SYSTEM_KRATOS.fmt(f),
            SystemNode::Sumsub => SYSTEM_SUMSUB.fmt(f),
        }
    }
}

#[derive(Clone, Copy, Debug, strum::EnumDiscriminants, Serialize, Deserialize)]
#[strum_discriminants(derive(strum::AsRefStr, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum Subject {
    Customer(CustomerId),
    User(UserId),
    System(SystemNode),
}

impl Subject {
    pub const fn core() -> Self {
        Subject::System(SystemNode::Core)
    }

    pub const fn init() -> Self {
        Subject::System(SystemNode::Init)
    }

    pub const fn kratos() -> Self {
        Subject::System(SystemNode::Kratos)
    }

    pub const fn sumsub() -> Self {
        Subject::System(SystemNode::Sumsub)
    }
}

impl std::str::FromStr for Subject {
    type Err = ParseSubjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(ParseSubjectError::InvalidSubjectFormat);
        }

        let id = parts[1].parse()?;
        use SubjectDiscriminants::*;
        let res = match SubjectDiscriminants::from_str(parts[0])? {
            Customer => Subject::Customer(CustomerId::from(id)),
            User => Subject::User(UserId::from(id)),
            System => match id {
                SYSTEM_INIT => Subject::System(SystemNode::Init),
                SYSTEM_CORE => Subject::System(SystemNode::Core),
                SYSTEM_KRATOS => Subject::System(SystemNode::Kratos),
                SYSTEM_SUMSUB => Subject::System(SystemNode::Sumsub),
                _ => return Err(ParseSubjectError::UnknownSystemNodeId(id)),
            },
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
    #[error("ParseSubjectError - UnknownSystemNodeId: {0}")]
    UnknownSystemNodeId(uuid::Uuid),
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
            Subject::System(id) => match id {
                SystemNode::Init => SYSTEM_INIT,
                SystemNode::Core => SYSTEM_CORE,
                SystemNode::Kratos => SYSTEM_KRATOS,
                SystemNode::Sumsub => SYSTEM_SUMSUB,
            },
        };
        write!(f, "{}:{}", SubjectDiscriminants::from(self).as_ref(), id)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SystemNode {
    Init,
    Core,
    Kratos,
    Sumsub,
}

const SYSTEM_INIT: Uuid = uuid!("00000000-0000-0000-0000-000000000001");
const SYSTEM_CORE: Uuid = uuid!("00000000-0000-0000-0000-000000000002");
const SYSTEM_KRATOS: Uuid = uuid!("00000000-0000-0000-0000-000000000003");
const SYSTEM_SUMSUB: Uuid = uuid!("00000000-0000-0000-0000-000000000004");

impl From<&Subject> for uuid::Uuid {
    fn from(s: &Subject) -> Self {
        match s {
            Subject::Customer(id) => uuid::Uuid::from(id),
            Subject::User(id) => uuid::Uuid::from(id),
            Subject::System(node) => match node {
                SystemNode::Init => SYSTEM_INIT,
                SystemNode::Core => SYSTEM_CORE,
                SystemNode::Kratos => SYSTEM_KRATOS,
                SystemNode::Sumsub => SYSTEM_SUMSUB,
            },
        }
    }
}
