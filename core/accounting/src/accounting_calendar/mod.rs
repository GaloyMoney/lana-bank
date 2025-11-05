mod entity;

pub mod error;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;

use crate::primitives::{CoreAccountingAction, CoreAccountingObject};

pub(super) use repo::*;

pub struct AccountingCalendars<Perms>
where
    Perms: PermissionCheck,
{
    repo: AccountingCalendarRepo,
    authz: Perms,
}

impl<Perms> Clone for AccountingCalendars<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
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
    pub fn new(pool: &sqlx::PgPool, authz: &Perms) -> Self {
        let repo = AccountingCalendarRepo::new(pool);

        Self {
            repo,
            authz: authz.clone(),
        }
    }
}
