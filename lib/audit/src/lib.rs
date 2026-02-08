#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, marker::PhantomData, str::FromStr};

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

    pub const BOOTSTRAP: Self = Self::new("bootstrap");
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

impl From<String> for SystemActor {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
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
