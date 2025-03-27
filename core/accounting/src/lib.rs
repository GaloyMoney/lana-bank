#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod chart_of_accounts;
pub mod general_ledger;
mod primitives;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;

pub use chart_of_accounts::*;
pub use primitives::*;

pub struct CoreAccounting<Perms>
where
    Perms: PermissionCheck,
{
    chart_of_accounts: ChartOfAccounts<Perms>,
}

impl<Perms> Clone for CoreAccounting<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            chart_of_accounts: self.chart_of_accounts.clone(),
        }
    }
}

impl<Perms> CoreAccounting<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: LedgerJournalId,
    ) -> Self {
        let chart_of_accounts = ChartOfAccounts::new(pool, authz, cala, journal_id);
        Self { chart_of_accounts }
    }

    pub fn chart_of_accounts(&self) -> &ChartOfAccounts<Perms> {
        &self.chart_of_accounts
    }
}
