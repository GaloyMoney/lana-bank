pub mod entity;
mod error;
mod ledger;
mod period;
mod repo;

use cala_ledger::CalaLedger;
use chrono::{DateTime, Utc};

use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::Idempotent;

use crate::primitives::{ChartId, CoreAccountingAction, CoreAccountingObject};

use entity::AccountingPeriod;
use error::AccountingPeriodError;
use ledger::AccountingPeriodLedger;
use repo::AccountingPeriodRepo;

pub struct AccountingPeriods<Perms>
where
    Perms: PermissionCheck,
{
    repo: AccountingPeriodRepo,
    ledger: AccountingPeriodLedger,
    authz: Perms,
}

impl<Perms> AccountingPeriods<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, cala: &CalaLedger) -> Self {
        Self {
            repo: AccountingPeriodRepo::new(pool),
            ledger: AccountingPeriodLedger::new(cala),
            authz: authz.clone(),
        }
    }

    /// Closes currently open monthly Accounting Period under the given
    /// Chart of Accounts and returns next Accounting Period.
    /// Fails if no such Accounting Period is found.
    pub async fn close_month(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        closed_at: DateTime<Utc>,
        chart_id: ChartId,
    ) -> Result<AccountingPeriod, AccountingPeriodError> {
        // TODO: check perms

        let mut open_periods = self.repo.find_open_accounting_periods(chart_id).await?;

        let open_period = open_periods
            .iter_mut()
            .find(|p| p.is_monthly())
            .ok_or(AccountingPeriodError::NoOpenAccountingPeriodFound)?;

        match open_period.close(closed_at, None)? {
            Idempotent::Executed(new) => {
                let mut db = self.repo.begin_op().await?;

                self.repo.update_in_op(&mut db, open_period).await?;
                let new_period = self.repo.create_in_op(&mut db, new).await?;
                self.ledger
                    .update_close_metadata_in_op(
                        db,
                        open_period.tracking_account_set,
                        open_period.period_end(),
                    )
                    .await?;

                Ok(new_period)
            }
            Idempotent::Ignored => Err(AccountingPeriodError::PeriodAlreadyClosed),
        }
    }

    /// Closes currently open annual Accounting Period under the given
    /// Chart of Accounts and returns next Accounting Period.
    /// Fails if no such Accounting Period is found.
    ///
    /// This method does not automatically close any other underlying
    /// Accounting Period.
    pub async fn close_year(&self, chart_id: &ChartId) -> Result<AccountingPeriod, String> {
        todo!()
    }
}
