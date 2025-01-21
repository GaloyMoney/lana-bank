#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod auth;
pub mod error;
mod primitives;
mod statement;

use cala_ledger::CalaLedger;

use audit::AuditSvc;
use authz::PermissionCheck;

pub use auth::*;
use error::*;
pub use primitives::*;

pub struct CoreStatements<Perms>
where
    Perms: PermissionCheck,
{
    cala: CalaLedger,
    authz: Perms,
    journal_id: LedgerJournalId,
}

impl<Perms> Clone for CoreStatements<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            cala: self.cala.clone(),
            authz: self.authz.clone(),
            journal_id: self.journal_id,
        }
    }
}

impl<Perms> CoreStatements<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreStatementsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreStatementsObject>,
{
    pub async fn init(
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: LedgerJournalId,
    ) -> Result<Self, CoreStatementsError> {
        let res = Self {
            cala: cala.clone(),
            authz: authz.clone(),
            journal_id,
        };
        Ok(res)
    }
}
