pub mod chart_of_accounts_integration;
pub mod entity;
pub mod error;

mod closing;
mod ledger;
mod period;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{AccountSetId, CalaLedger};
use chart_of_accounts_integration::{AccountingPeriodConfig, Basis};
use chrono::NaiveDate;
use es_entity::Idempotent;
use tracing::instrument;

use crate::{
    Chart,
    chart_of_accounts::ChartOfAccounts,
    primitives::{
        AccountingPeriodId, CalaJournalId, CalaTxId, ChartId, CoreAccountingAction,
        CoreAccountingObject,
    },
};

pub use chart_of_accounts_integration::ChartOfAccountsIntegrationConfig;
pub use entity::AccountingPeriod;
#[cfg(feature = "json-schema")]
pub use entity::AccountingPeriodEvent;
pub(super) use entity::*;
use error::AccountingPeriodError;
pub(crate) use ledger::ClosingMetadata;
use ledger::{
    AccountingPeriodAccountSetIds, AccountingPeriodLedger, ChartOfAccountsIntegrationMeta,
};
pub use period::Period;
use repo::AccountingPeriodRepo;

pub struct AccountingPeriods<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    repo: AccountingPeriodRepo,
    ledger: AccountingPeriodLedger,
    chart_of_accounts: ChartOfAccounts<Perms>,
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
        chart_of_accounts: &ChartOfAccounts<Perms>,
    ) -> Self {
        Self {
            authz: authz.clone(),
            repo: AccountingPeriodRepo::new(pool),
            ledger: AccountingPeriodLedger::new(cala, journal_id),
            chart_of_accounts: chart_of_accounts.clone(),
        }
    }

    /// Generates first Accounting Periods according to their
    /// configurations. The periods will be created in such a way that
    /// they are open on `date`. If any Accounting Periods already
    /// exist, no new periods are added.
    pub async fn open_initial_periods(
        &self,
        chart_id: ChartId,
        tracking_account_set_id: AccountSetId,
        date: NaiveDate,
        periods: Vec<AccountingPeriodConfig>,
    ) -> Result<(), AccountingPeriodError> {
        let open_periods = self.repo.find_open_accounting_periods(chart_id).await?;

        let chart = self.chart_of_accounts.find_by_id(chart_id).await?;

        let chart_config = self
            .ledger
            .get_chart_of_accounts_integration_config(tracking_account_set_id)
            .await?
            .ok_or(AccountingPeriodError::AccountingPeriodIntegrationConfigNotFound)?;

        let account_set_ids = AccountingPeriodAccountSetIds {
            tracking_account_set_id,
            revenue_account_set_id: chart.account_set_id_from_code(&chart_config.revenue_code)?,
            cost_of_revenue_account_set_id: chart
                .account_set_id_from_code(&chart_config.cost_of_revenue_code)?,
            expenses_account_set_id: chart.account_set_id_from_code(&chart_config.expenses_code)?,
            equity_retained_earnings_account_set_id: chart
                .account_set_id_from_code(&chart_config.equity_retained_earnings_code)?,
            equity_retained_losses_account_set_id: chart
                .account_set_id_from_code(&chart_config.equity_retained_losses_code)?,
        };

        if open_periods.is_empty() {
            for period in periods {
                let period = match period.basis {
                    Basis::Monthly { day } => {
                        Period::monthly_around_date(day, date, period.grace_period_days).unwrap()
                    }
                    Basis::Annual { day, month } => {
                        Period::annually_around_date(day, month, date, period.grace_period_days)
                            .unwrap()
                    }
                };

                self.repo
                    .create(NewAccountingPeriod {
                        id: AccountingPeriodId::new(),
                        chart_id,
                        period,
                        account_set_ids,
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
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
    ) -> Result<AccountingPeriod, AccountingPeriodError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_period(),
                CoreAccountingAction::ACCOUNTING_PERIOD_CLOSE,
            )
            .await?;
        let mut open_periods = self.repo.find_open_accounting_periods(chart_id).await?;

        let open_period = open_periods
            .iter_mut()
            .find(|p| p.is_monthly())
            .ok_or(AccountingPeriodError::NoOpenAccountingPeriodFound)?;
        let closed_at = crate::time::now();
        match open_period.close(closed_at, None)? {
            Idempotent::Executed(new) => {
                let mut db = self.repo.begin_op().await?;

                self.repo.update_in_op(&mut db, open_period).await?;
                let new_period = self.repo.create_in_op(&mut db, new).await?;
                self.ledger
                    .update_close_metadata_in_op(
                        db,
                        open_period.account_set_ids.tracking_account_set_id,
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
    /// This method closes all other Accounting Periods in an
    /// unspecified order.
    pub async fn close_year(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        description: Option<String>,
    ) -> Result<AccountingPeriod, AccountingPeriodError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_period(),
                CoreAccountingAction::ACCOUNTING_PERIOD_CLOSE,
            )
            .await?;

        let (mut open_annual_period, remaining_open_periods) = {
            let mut open_periods = self.repo.find_open_accounting_periods(chart_id).await?;

            let open_annual_period_index = open_periods
                .iter()
                .position(|p| p.is_annual())
                .ok_or(AccountingPeriodError::NoOpenAccountingPeriodFound)?;

            let open_annual_period = open_periods.remove(open_annual_period_index);
            (open_annual_period, open_periods)
        };

        let closed_at = crate::time::now();
        let ledger_tx_id = CalaTxId::new();
        match open_annual_period.close(closed_at, Some(ledger_tx_id))? {
            Idempotent::Executed(new) => {
                let mut db = self.repo.begin_op().await?;

                self.repo
                    .update_in_op(&mut db, &mut open_annual_period)
                    .await?;
                let new_period = self.repo.create_in_op(&mut db, new).await?;

                for mut period in remaining_open_periods {
                    if period.close_unchecked(closed_at, None).did_execute() {
                        self.repo.update_in_op(&mut db, &mut period).await?;
                    }
                }

                self.ledger
                    .close_year_in_op(db, ledger_tx_id, description, open_annual_period)
                    .await?;
                Ok(new_period)
            }
            Idempotent::Ignored => Err(AccountingPeriodError::PeriodAlreadyClosed),
        }
    }

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        chart: &Chart,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, AccountingPeriodError> {
        Ok(self
            .ledger
            .get_chart_of_accounts_integration_config(chart.account_set_id)
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
            .get_chart_of_accounts_integration_config(chart)
            .await?
            .is_some()
        {
            return Err(AccountingPeriodError::AccountingPeriodIntegrationConfigAlreadyExists);
        }

        let revenue_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.revenue_code)?;
        let cost_of_revenue_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.cost_of_revenue_code)?;
        let expenses_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.expenses_code)?;
        let equity_retained_earnings_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.equity_retained_earnings_code)?;
        let equity_retained_losses_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.equity_retained_losses_code)?;

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

    #[instrument(name = "core_accounting.accounting_periods.find_all", skip(self), err)]
    pub async fn find_all<T: From<AccountingPeriod>>(
        &self,
        ids: &[AccountingPeriodId],
    ) -> Result<std::collections::HashMap<AccountingPeriodId, T>, AccountingPeriodError> {
        self.repo.find_all(ids).await
    }
}
