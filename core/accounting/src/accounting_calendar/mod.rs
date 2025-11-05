mod entity;

pub mod error;
pub mod ledger;
mod repo;

use es_entity::Idempotent;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

use cala_ledger::CalaLedger;

use crate::primitives::{
    AccountingCalendarId, CalaAccountSetId, CoreAccountingAction, CoreAccountingObject,
};

pub(super) use entity::*;
use error::*;
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

    #[instrument(
        name = "core_accounting.accounting_calendar.close_monthly",
        skip(self,),
        err
    )]
    pub async fn close_monthly(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<AccountingCalendarId> + std::fmt::Debug,
        tracking_account_set_id: impl Into<CalaAccountSetId> + std::fmt::Debug,
    ) -> Result<AccountingCalendar, AccountingCalendarError> {
        let id = id.into();
        let mut calendar = self.repo.find_by_id(id).await?;

        let now = crate::time::now();
        let closed_as_of_date =
            if let Idempotent::Executed(date) = calendar.close_last_monthly_period(now)? {
                date
            } else {
                return Ok(calendar);
            };

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut calendar).await?;

        self.accounting_calendar_ledger
            .monthly_close_chart_as_of(op, tracking_account_set_id, closed_as_of_date)
            .await?;

        Ok(calendar)
    }
}
