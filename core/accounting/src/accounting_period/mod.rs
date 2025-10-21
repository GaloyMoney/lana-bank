pub mod chart_of_accounts_integration;
pub mod entity;
pub mod error;

mod ledger;
mod period;
mod primitives;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{AccountSetId, CalaLedger};
use es_entity::Idempotent;
use rust_decimal::Decimal;
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
    AccountingPeriodLedger, ChartOfAccountsIntegrationMeta, ClosingTransactionParams, EntryParams,
};
pub use period::Period;
use primitives::ProfitAndLossClosingDetails;
use repo::AccountingPeriodRepo;

pub struct AccountingPeriods<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    repo: AccountingPeriodRepo,
    ledger: AccountingPeriodLedger,
    journal_id: CalaJournalId,
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
        let ledger = AccountingPeriodLedger::new(cala);
        Self {
            authz: authz.clone(),
            repo: repo.clone(),
            ledger: ledger.clone(),
            chart_of_accounts: chart_of_accounts.clone(),
            journal_id,
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
        let parent_profit_account_set_id = self
            .chart_of_accounts
            .find_account_set_id_by_code(chart_id, chart_config.equity_retained_earnings_code)
            .await?;
        let parent_losses_account_set_id = self
            .chart_of_accounts
            .find_account_set_id_by_code(chart_id, chart_config.equity_retained_losses_code)
            .await?;
        let period_end_balances = self
            .chart_of_accounts
            .get_profit_and_loss_statement_closing_balances(
                chart_id,
                open_annual_period.period_end(),
                chart_config.revenue_code,
                chart_config.cost_of_revenue_code,
                chart_config.expenses_code,
            )
            .await?;
        let revenue_closing_details = self
            .ledger
            .create_closing_offset_entries(description.clone(), period_end_balances.revenue);
        let expense_closing_details = self
            .ledger
            .create_closing_offset_entries(description.clone(), period_end_balances.expenses);
        let cost_of_revenue_closing_details = self.ledger.create_closing_offset_entries(
            description.clone(),
            period_end_balances.cost_of_revenue,
        );
        let net_income = self.calculate_net_income(
            &revenue_closing_details,
            &expense_closing_details,
            &cost_of_revenue_closing_details,
        );
        let mut tx_entries = Vec::new();
        tx_entries.extend(revenue_closing_details.closing_entries);
        tx_entries.extend(expense_closing_details.closing_entries);
        tx_entries.extend(cost_of_revenue_closing_details.closing_entries);
        let entry_params = tx_entries.into_iter().map(EntryParams::from).collect();

        let ledger_tx_id = CalaTxId::new();
        match open_annual_period.close(effective, Some(ledger_tx_id))? {
            Idempotent::Executed(new) => {
                let mut db = self.repo.begin_op().await?;
                self.repo.update_in_op(&mut db, open_annual_period).await?;
                let new_period = self.repo.create_in_op(&mut db, new).await?;
                self.ledger
                    .execute_closing(
                        db,
                        net_income,
                        parent_profit_account_set_id,
                        parent_losses_account_set_id,
                        ledger_tx_id,
                        ClosingTransactionParams {
                            journal_id: self.journal_id,
                            description: description.unwrap_or("Closing Entry".to_string()),
                            effective: effective.date_naive(),
                            entry_params,
                        },
                    )
                    .await?;
                Ok(new_period)
            }
            Idempotent::Ignored => Err(AccountingPeriodError::PeriodAlreadyClosed),
        }
    }

    fn calculate_net_income(
        &self,
        revenue_details: &ProfitAndLossClosingDetails,
        expense_details: &ProfitAndLossClosingDetails,
        cost_of_revenue_details: &ProfitAndLossClosingDetails,
    ) -> Decimal {
        revenue_details.net_category_balance
            - expense_details.net_category_balance
            - cost_of_revenue_details.net_category_balance
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
