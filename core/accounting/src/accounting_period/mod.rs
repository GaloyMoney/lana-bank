pub mod entity;
mod error;
mod period;
mod ledger;
mod repo;

use chrono::{DateTime, Utc};

use entity::{AccountingPeriod, NewAccountingPeriod};
use error::AccountingPeriodError;
use es_entity::Idempotent;
use repo::AccountingPeriodRepo;
use ledger::AccountingPeriodLedger;
use crate::{
    AccountingPeriodId, CalaJournalId, ChartId,
};

pub struct AccountingPeriods {
    repo: AccountingPeriodRepo,
    ledger: AccountingPeriodLedger,
    journal_id: CalaJournalId,
}

impl AccountingPeriods {
    /// Closes currently open monthly Accounting Period under the given
    /// Chart of Accounts and returns next Accounting Period.
    /// Fails if no such Accounting Period is found.
    pub async fn close_month_in_op(
        &self,
        mut db: es_entity::DbOp<'_>,
        closed_at: DateTime<Utc>,
        chart_id: ChartId,
    ) -> Result<AccountingPeriod, AccountingPeriodError> {
        let mut open_periods = self.repo.find_open_accounting_periods(chart_id).await?;

        let pos = open_periods
            .iter()
            .position(|p| p.is_monthly())
            .ok_or(AccountingPeriodError::NoOpenAccountingPeriodFound)?;

        let now = crate::time::now();
        let mut open_period = open_periods.remove(pos);
        match open_period.close(closed_at, None) {
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
        _period: &AccountingPeriod,
    ) -> Result<(), AccountingPeriodError> {
        let closed_as_of = closed_as_of.date_naive();
        self.ledger
            .update_close_metadata(db, chart_id, closed_as_of)
            .await?;
        Ok(())
    }
}
