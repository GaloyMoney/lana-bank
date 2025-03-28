#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod chart_of_accounts;
pub mod journal;
pub mod ledger_account;
mod primitives;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;

pub use chart_of_accounts::{Chart, ChartOfAccounts, error as chart_of_accounts_error, tree};
pub use journal::{Journal, error as journal_error};
pub use ledger_account::LedgerAccounts;
pub use primitives::*;

pub struct CoreAccounting<Perms>
where
    Perms: PermissionCheck,
{
    chart_of_accounts: ChartOfAccounts<Perms>,
    journal: Journal<Perms>,
    ledger_accounts: LedgerAccounts<Perms>,
}

impl<Perms> Clone for CoreAccounting<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            chart_of_accounts: self.chart_of_accounts.clone(),
            journal: self.journal.clone(),
            ledger_accounts: self.ledger_accounts.clone(),
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
        let journal = Journal::new(authz, cala, journal_id);
        let ledger_accounts = LedgerAccounts::new(authz, cala, journal_id);
        Self {
            chart_of_accounts,
            journal,
            ledger_accounts,
        }
    }

    pub fn chart_of_accounts(&self) -> &ChartOfAccounts<Perms> {
        &self.chart_of_accounts
    }

    pub fn journal(&self) -> &Journal<Perms> {
        &self.journal
    }

    pub fn ledger_accounts(&self) -> &LedgerAccounts<Perms> {
        &self.ledger_accounts
    }
}
