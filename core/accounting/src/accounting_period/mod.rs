pub mod entity;
mod error;
mod repo;

use std::collections::HashMap;

use entity::{AccountingPeriod, NewAccountingPeriod};
use error::AccountingPeriodError;
use es_entity::Idempotent;
use repo::AccountingPeriodRepo;

use crate::{AccountingPeriodId, ChartId};

struct AccountingPeriods {
    repo: AccountingPeriodRepo,
}

impl AccountingPeriods {
    /// Returns a list of all Accounting Periods that are currently
    /// open on the given chart. No specific order of the periods is
    /// guaranteed.
    pub async fn find_open_accounting_periods(
        &self,
        chart_id: ChartId,
    ) -> Result<HashMap<AccountingPeriodId, AccountingPeriod>, AccountingPeriodError> {
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
            .find_map(|(id, p)| {
                if p.frequency.is_monthly() {
                    Some(*id)
                } else {
                    None
                }
            })
            .ok_or(AccountingPeriodError::NoOpenAccountingPeriodFound)?;

        let mut open_period = open_periods
            .remove(&id)
            .expect("Value has been confirmed to be present.");

        match open_period.close(None)? {
            Idempotent::Executed(new) => {
                self.repo.update_in_op(&mut db, &mut open_period).await?;
                let new_period = self.repo.create_in_op(&mut db, new).await?;
                self.somehow_update_metadata_in_op(db, &open_period).await;
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

    async fn somehow_update_metadata_in_op(
        &self,
        db: es_entity::DbOp<'_>,
        period: &AccountingPeriod,
    ) {
        todo!()
    }
}
