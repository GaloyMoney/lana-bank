#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, marker::PhantomData, str::FromStr};
use uuid::Uuid;

pub mod error;
mod primitives;
mod svc_trait;

pub use primitives::*;
pub use svc_trait::*;

/// Identifies the specific system actor performing an operation.
/// Used to differentiate between external integrations, internal jobs, and system operations.
///
/// Each `core/` module defines its own constants (e.g. `core_credit::primitives::INTEREST_ACCRUAL`).
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SystemActor(Cow<'static, str>);

impl SystemActor {
    pub const fn new(actor: &'static str) -> Self {
        Self(Cow::Borrowed(actor))
    }
}

impl fmt::Display for SystemActor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for SystemActor {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for SystemActor {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Cow::Owned(s.to_string())))
    }
}

/// Represents who initiated an operation (user, customer, or system actor).
/// This type can be used across core and lana modules.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Subject {
    System(SystemActor),
    User(Uuid),
    Customer(Uuid),
}

#[derive(Debug, thiserror::Error)]
pub enum SubjectParseError {
    #[error("invalid uuid: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("unknown subject format: {0}")]
    UnknownFormat(String),
}

impl tracing_utils::ErrorSeverity for SubjectParseError {
    fn severity(&self) -> tracing::Level {
        tracing::Level::ERROR
    }
}

impl Subject {
    /// Parse a subject from a string representation (e.g., "system:bootstrap", "user:uuid", "customer:uuid")
    pub fn try_from_string(s: &str) -> Result<Self, SubjectParseError> {
        if let Some(actor_str) = s.strip_prefix("system:") {
            return Ok(Self::System(actor_str.parse().unwrap()));
        }

        if let Some(id_str) = s.strip_prefix("user:") {
            let id = Uuid::parse_str(id_str)?;
            return Ok(Self::User(id));
        }

        if let Some(id_str) = s.strip_prefix("customer:") {
            let id = Uuid::parse_str(id_str)?;
            return Ok(Self::Customer(id));
        }

        Err(SubjectParseError::UnknownFormat(s.to_string()))
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Subject::System(actor) => write!(f, "system:{}", actor.as_ref()),
            Subject::User(id) => write!(f, "user:{}", id),
            Subject::Customer(id) => write!(f, "customer:{}", id),
        }
    }
}

impl FromStr for Subject {
    type Err = SubjectParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_string(s)
    }
}

impl SystemSubject for Subject {
    fn system(actor: SystemActor) -> Self {
        Subject::System(actor)
    }
}

// Re-export pagination types for consumers who need them
pub use es_entity::{PaginatedQueryArgs, PaginatedQueryRet};

#[derive(Clone)]
pub struct Audit<S, O, A> {
    pool: sqlx::PgPool,
    _subject: PhantomData<S>,
    _object: PhantomData<O>,
    _action: PhantomData<A>,
}

impl<S, O, A> Audit<S, O, A> {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        Self {
            pool: pool.clone(),
            _subject: std::marker::PhantomData,
            _object: std::marker::PhantomData,
            _action: std::marker::PhantomData,
        }
    }
}

impl<S, O, A> AuditSvc for Audit<S, O, A>
where
    S: FromStr + fmt::Display + fmt::Debug + Clone + Send + Sync + SystemSubject + 'static,
    O: FromStr + fmt::Display + fmt::Debug + Copy + Send + Sync + 'static,
    A: FromStr + fmt::Display + fmt::Debug + Copy + Send + Sync + 'static,
{
    type Subject = S;
    type Object = O;
    type Action = A;

    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
