pub mod chart_of_accounts_integration;
pub mod entity;
pub mod error;

mod ledger;
mod period;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{AccountSetId, CalaLedger, JournalId};
use chrono::{DateTime, Utc};
use es_entity::Idempotent;
use period::Period;
use tracing::instrument;

use crate::{
    Chart,
    primitives::{
        AccountingPeriodId, CalaJournalId, ChartId, CoreAccountingAction, CoreAccountingObject,
    },
};
pub use chart_of_accounts_integration::ChartOfAccountsIntegrationConfig;
use ledger::{AccountingPeriodLedger, ChartOfAccountsIntegrationMeta};

use error::*;
use repo::*;

pub use entity::AccountingPeriod;
#[cfg(feature = "json-schema")]
pub use entity::AccountingPeriodEvent;
pub(super) use entity::*;

#[derive(Clone)]
pub struct AccountingPeriods<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    repo: AccountingPeriodRepo,
    ledger: AccountingPeriodLedger,
    journal_id: CalaJournalId,
}

impl<Perms> AccountingPeriods<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        authz: &Perms,
        pool: &sqlx::PgPool,
        cala: &CalaLedger,
        journal_id: CalaJournalId,
    ) -> Self {
        let repo = AccountingPeriodRepo::new(pool);
        let ledger = AccountingPeriodLedger::new(cala, journal_id);
        Self {
            authz: authz.clone(),
            repo: repo.clone(),
            ledger: ledger.clone(),
            journal_id,
        }
    }

    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
            journal_id: self.journal_id,
        }
    }

    /// Generates Accounting Periods to initialize their cycle. If any
    /// Accounting Periods already exist, no new periods are added.
    pub async fn open_initial_periods(
        &self,
        chart_id: ChartId,
        tracking_account_set: AccountSetId,
        periods: Vec<Period>,
    ) -> Result<(), AccountingPeriodError> {
        let open_periods = self.repo.find_open_accounting_periods(chart_id).await?;

        if open_periods.is_empty() {
            for period in periods {
                self.repo
                    .create(NewAccountingPeriod {
                        id: AccountingPeriodId::new(),
                        chart_id,
                        tracking_account_set,
                        period,
                        closed_at: None,
                    })
                    .await?;
            }
        }

        Ok(())
    }

    /// Closes currently open monthly Accounting Period under the given
    /// Chart of Accounts and returns next Accounting Period.
    /// Fails if no such Accounting Period is found.
    pub async fn close_month(
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
        // TODO: Return or fetch from repo?
        let now = crate::time::now();
        let mut open_period = open_periods.remove(pos);
        match open_period.close(closed_at, None) {
            Idempotent::Executed(new) => {
                self.repo.update_in_op(&mut db, &mut open_period).await?;
                let new_period = self.repo.create_in_op(&mut db, new).await?;
                self.update_close_metadata(db, chart_id, now).await?;
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
    ) -> Result<(), AccountingPeriodError> {
        let closed_as_of = closed_as_of.date_naive();
        self.ledger
            .update_close_metadata(db, chart_id, closed_as_of)
            .await?;
        Ok(())
    }
    pub async fn get_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, AccountingPeriodError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_period_configuration(),
                CoreAccountingAction::ACCOUNTING_PERIOD_CONFIGURATION_READ,
            )
            .await?;

        Ok(self
            .ledger
            .get_chart_of_accounts_integration_config(chart.id)
            .await?)
    }

    pub async fn set_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, AccountingPeriodError> {
        if chart.id != config.chart_of_accounts_id {
            return Err(AccountingPeriodError::ChartIdMismatch);
        }

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_period_configuration(),
                CoreAccountingAction::ACCOUNTING_PERIOD_CONFIGURATION_UPDATE,
            )
            .await?;

        if self
            .ledger
            .get_chart_of_accounts_integration_config(chart.id)
            .await?
            .is_some()
        {
            return Err(AccountingPeriodError::AccountingPeriodIntegrationConfigAlreadyExists);
        }

        let revenue_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.chart_of_accounts_revenue_code)?;
        let cost_of_revenue_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.chart_of_accounts_cost_of_revenue_code)?;
        let expenses_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.chart_of_accounts_expenses_code)?;
        let equity_retained_earnings_child_account_set_id_from_chart = chart
            .account_set_id_from_code(&config.chart_of_accounts_equity_retained_earnings_code)?;
        let equity_retained_losses_child_account_set_id_from_chart = chart
            .account_set_id_from_code(&config.chart_of_accounts_equity_retained_losses_code)?;

        let charts_integration_meta = ChartOfAccountsIntegrationMeta {
            audit_info,
            config: config.clone(),

            revenue_child_account_set_id_from_chart,
            cost_of_revenue_child_account_set_id_from_chart,
            expenses_child_account_set_id_from_chart,
            equity_retained_earnings_child_account_set_id_from_chart,
            equity_retained_losses_child_account_set_id_from_chart,
        };

        let db = self.repo.begin_op().await?;
        self.ledger
            .attach_chart_of_accounts_integration_meta(db, chart.id, charts_integration_meta)
            .await?;

        Ok(config)
    }
}
