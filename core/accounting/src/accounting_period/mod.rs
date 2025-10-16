pub mod entity;
mod error;
mod period;
mod ledger;
mod repo;

use std::collections::HashMap;
use chrono::{DateTime, Utc};

use audit::AuditSvc;
use authz::PermissionCheck;

use entity::{AccountingPeriod, NewAccountingPeriod};
use error::AccountingPeriodError;
use es_entity::Idempotent;
use repo::AccountingPeriodRepo;
use ledger::AccountingPeriodLedger;
use cala_ledger::CalaLedger;
use crate::{
    AccountingPeriodId, CalaJournalId, ChartId, CoreAccountingAction, CoreAccountingObject
};

pub struct AccountingPeriods<Perms>
where
    Perms: PermissionCheck,
{
    repo: AccountingPeriodRepo,
    ledger: AccountingPeriodLedger,
    journal_id: CalaJournalId,
    authz: Perms,
}

impl<Perms> Clone for AccountingPeriods<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
            journal_id: self.journal_id,
        }
    }
}

impl<Perms> AccountingPeriods<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(repo: AccountingPeriodRepo, authz: &Perms, cala: &CalaLedger, journal_id: CalaJournalId) -> Self {
        Self {
            repo,
            authz: authz.clone(),
            ledger: AccountingPeriodLedger::new(cala, journal_id),
            journal_id,
        }
    }

    /// Returns a list of all Accounting Periods that are currently
    /// open on the given chart. No specific order of the periods is
    /// guaranteed.
    pub async fn find_open_accounting_periods(
        &self,
        chart_id: ChartId,
    ) -> Result<HashMap<AccountingPeriodId, AccountingPeriod>, AccountingPeriodError> {
        // TODO: Discuss a stopping assumption - once we found a `AccountingPeriodEvent::Closed` for the largest `Frequency`,
        // where we can assume there will be no more?
        // - `find_open_periods_in_current_year`?
        // let periods = self.repo.list_by_created_at(
        //     Default::default(),
        //     es_entity::ListDirection::Descending,
        // ).await?;

        // let mut open_by_id: HashMap<AccountingPeriodId, AccountingPeriod> = HashMap::new();
        // for p in periods.entities {
        //     if !p.is_closed() {
        //         open_by_id.insert(p.id, p);
        //     }
        // }
        todo!()
    }

    /// Closes currently open monthly Accounting Period under the given
    /// Chart of Accounts and returns next Accounting Period.
    /// Fails if no such Accounting Period is found.
    pub async fn close_month_in_op(
        &self,
        mut db: es_entity::DbOp<'_>,
        chart_id: ChartId,
    ) -> Result<AccountingPeriod, AccountingPeriodError> {
        let mut open_periods = self.find_open_accounting_periods(chart_id).await?;

        let id = open_periods
            .iter()
            .find_map(|(id, p)| if p.is_monthly() { Some(*id) } else { None })
            .ok_or(AccountingPeriodError::NoOpenAccountingPeriodFound)?;

        let mut open_period = open_periods
            .remove(&id)
            .expect("Value has been confirmed to be present.");

        let now = crate::time::now();
        match open_period.close(now, None) {
            Idempotent::Executed(new) => {
                self.repo.update_in_op(&mut db, &mut open_period).await?;
                let new_period = self.repo.create_in_op(&mut db, new).await?;
                self.update_close_metadata(db, chart_id, now, &open_period)
                    .await;
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

    async fn update_close_metadata(
        &self,
        db: es_entity::DbOp<'_>,
        chart_id: ChartId,
        closed_as_of: DateTime<Utc>,
        // TODO: Can we remove this parameter?
        _period: &AccountingPeriod,
    ) -> Result<(), AccountingPeriodError> {
        let closed_as_of = closed_as_of.date_naive();
        self.ledger
            .update_close_metadata(db, chart_id, closed_as_of)
            .await?;
        Ok(())
    }
}
