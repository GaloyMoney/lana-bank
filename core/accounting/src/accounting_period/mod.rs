pub mod chart_of_accounts_integration;
pub mod entity;
pub mod error;

mod ledger;
mod period;
mod closing;
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
use ledger::{
    AccountingPeriodLedger, ChartOfAccountsIntegrationMeta,
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
        let repo = AccountingPeriodRepo::new(pool);
        let ledger = AccountingPeriodLedger::new(cala, journal_id);
        Self {
            authz: authz.clone(),
            repo: repo.clone(),
            ledger: ledger.clone(),
            chart_of_accounts: chart_of_accounts.clone(),
        }
    }

    /// Generates Accounting Periods to initialize their cycle. If any
    /// Accounting Periods already exist, no new periods are added.
    pub async fn open_initial_periods(
        &self,
        chart_id: ChartId,
        tracking_account_set: AccountSetId,
        periods: Vec<AccountingPeriodConfig>,
    ) -> Result<(), AccountingPeriodError> {
        let open_periods = self.repo.find_open_accounting_periods(chart_id).await?;

        let today = crate::time::now().date_naive();

        if open_periods.is_empty() {
            for period in periods {
                let period = match period.basis {
                    Basis::Monthly { on_day } => {
                        Period::monthly_around_date(on_day, today, period.grace_period_days)
                            .unwrap()
                    }
                    Basis::Annual { on_month, on_day } => Period::annually_around_date(
                        on_day,
                        on_month,
                        today,
                        period.grace_period_days,
                    )
                    .unwrap(),
                };

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
        
        // TODO: Also need to verify that the related monthly period is still open 
        // (the one with the same `period_end`)?
        let mut open_periods = self.repo.find_open_accounting_periods(chart_id).await?;

        let open_annual_period = open_periods
            .iter_mut()
            .find(|p| p.is_annual())
            .ok_or(AccountingPeriodError::NoOpenAccountingPeriodFound)?;
        let chart_config = self
            .ledger
            .get_chart_of_accounts_integration_config(open_annual_period.tracking_account_set)
            .await?
            .ok_or(AccountingPeriodError::AccountingPeriodIntegrationConfigNotFound)?;

        let effective = crate::time::now();
        let retained_earnings_account_set_ids = self
            .chart_of_accounts
            .find_retained_earnings_account_sets_by_codes(
                chart_id,
                chart_config.equity_retained_earnings_code,
                chart_config.equity_retained_losses_code,
            )
            .await?;
        
        // TODO: Is there an existing primitive that can be used for this?
        let period_end_balances = self
            .chart_of_accounts
            .get_profit_and_loss_statement_period_end_balances(
                chart_id,
                open_annual_period.period_end(),
                chart_config.revenue_code,
                chart_config.cost_of_revenue_code,
                chart_config.expenses_code,
            )
            .await?;

        let ledger_tx_id = CalaTxId::new();
        match open_annual_period.close(effective, Some(ledger_tx_id))? {
            Idempotent::Executed(new) => {
                let mut db = self.repo.begin_op().await?;
                self.repo.update_in_op(&mut db, open_annual_period).await?;
                let new_period = self.repo.create_in_op(&mut db, new).await?;
                self.ledger
                    .execute_closing(
                        db,
                        ledger_tx_id,
                        description,
                        effective.date_naive(),
                        period_end_balances,
                        retained_earnings_account_set_ids,
                    )
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
