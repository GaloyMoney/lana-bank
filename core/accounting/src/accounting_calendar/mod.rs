mod entity;

pub mod error;
pub mod ledger;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;

use cala_ledger::CalaLedger;

use crate::primitives::{AccountingCalendarId, CoreAccountingAction, CoreAccountingObject};

use ledger::*;
pub(super) use repo::*;

pub struct AccountingCalendars<Perms>
where
    Perms: PermissionCheck,
{
    repo: AccountingCalendarRepo,
    accounting_calendar_ledger: AccountingCalendarLedger,
    authz: Perms,
}

impl<Perms> Clone for AccountingCalendars<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            accounting_calendar_ledger: self.accounting_calendar_ledger.clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms> AccountingCalendars<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, cala: &CalaLedger) -> Self {
        let repo = AccountingCalendarRepo::new(pool);
        let accounting_calendar_ledger = AccountingCalendarLedger::new(cala);

        Self {
            repo,
            accounting_calendar_ledger,
            authz: authz.clone(),
        }
    }
}
