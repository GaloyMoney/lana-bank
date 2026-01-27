#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use serde::{Deserialize, Serialize};
use std::{fmt, marker::PhantomData, str::FromStr};

pub mod error;
mod primitives;
mod svc_trait;

pub use primitives::*;
pub use svc_trait::*;

/// Identifies the specific system actor performing an operation.
/// Used to differentiate between external integrations, internal jobs, and CLI operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(strum::Display, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum SystemActor {
    // External integrations
    Sumsub,
    BitGo,
    Komainu,

    // Credit module jobs
    InterestAccrual,
    ObligationSync,
    CollateralizationSync,
    CreditFacilityJob,
    DisbursalJob,

    // Deposit module
    DepositSync,
    DepositApproval,

    // Custody module
    CustodyWebhook,

    // Customer module
    KycCallback,
    CustomerSync,

    // Accounting module
    AccountingJob,

    // Governance
    Governance,

    // System operations
    ReportsSync,
    Bootstrap,
    Cli,

    // Backward compatibility for existing audit entries
    Unknown,
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
    S: FromStr + fmt::Display + fmt::Debug + Clone + Sync + Send + SystemSubject + 'static,
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
