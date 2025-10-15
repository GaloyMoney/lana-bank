mod chart_of_accounts_integration;
mod entity;
pub mod error;
mod ledger;
mod primitives;
mod repo;
pub use chart_of_accounts_integration::ChartOfAccountsIntegrationConfig;
use tracing::instrument;

use crate::{
    chart_of_accounts::{ChartOfAccounts, Chart},
    primitives::{CoreAccountingAction, CoreAccountingObject, CalaAccountSetId},
};
use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{AccountSetId, CalaLedger, JournalId};
use error::*;
use ledger::{ChartOfAccountsIntegrationMeta, ClosingLedger, EntryParams};

pub use entity::LedgerClosing;
#[cfg(feature = "json-schema")]
pub use entity::LedgerClosingEvent;
pub(super) use entity::*;
pub use repo::ledger_closing_cursor::LedgerClosingsByCreatedAtCursor;
use repo::*;

pub(crate) const REVENUE_NAME: &str = "Revenue";
pub(crate) const EXPENSES_NAME: &str = "Expenses";
pub(crate) const COST_OF_REVENUE_NAME: &str = "Cost of Revenue";
pub(crate) const EQUITY_RETAINED_EARNINGS_NAME: &str = "Retained Earnings";
pub(crate) const EQUITY_RETAINED_LOSSES_NAME: &str = "Retained Losses";

#[derive(Clone, Copy)]
pub struct AnnualClosingIds {
    pub id: CalaAccountSetId,
    pub revenue: CalaAccountSetId,
    pub cost_of_revenue: CalaAccountSetId,
    pub expenses: CalaAccountSetId,
    pub equity_retained_earnings: CalaAccountSetId,
    pub equity_retained_losses: CalaAccountSetId,
}

#[derive(Clone)]
pub struct LedgerClosings<Perms>
where
    Perms: PermissionCheck,
{
    ledger: ClosingLedger,
    authz: Perms,
    chart_of_accounts: ChartOfAccounts<Perms>,
    journal_id: JournalId,
    repo: LedgerClosingRepo,
}

impl<Perms> LedgerClosings<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        chart_of_accounts: &ChartOfAccounts<Perms>,
        journal_id: JournalId,
    ) -> Self {
        let repo = LedgerClosingRepo::new(pool);
        Self {
            ledger: ClosingLedger::new(cala),
            authz: authz.clone(),
            journal_id,
            chart_of_accounts: chart_of_accounts.clone(),
            repo,
        }
    }

    #[instrument(
        name = "core_accounting.ledger_closings.close_last_period",
        skip(self),
        err
    )]
    pub async fn close_last_period(&self) -> Result<(), LedgerClosingError> {
        todo!()
    }

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_root_account_set_id: CalaAccountSetId,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, LedgerClosingError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_ledger_closing_configuration(),
                CoreAccountingAction::LEDGER_CLOSING_CONFIGURATION_READ,
            )
            .await?;

        Ok(self
            .ledger
            .get_chart_of_accounts_integration_config(chart_root_account_set_id)
            .await?)
    }

    pub async fn set_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, LedgerClosingError> {
        if chart.id != config.chart_of_accounts_id {
            return Err(LedgerClosingError::ChartIdMismatch);
        }

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_ledger_closing_configuration(),
                CoreAccountingAction::LEDGER_CLOSING_CONFIGURATION_UPDATE,
            )
            .await?;

        if self
            .ledger
            .get_chart_of_accounts_integration_config(chart.id)
            .await?
            .is_some()
        {
            return Err(LedgerClosingError::LedgerClosingIntegrationConfigAlreadyExists);
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

    async fn execute_monthly_closing_operation(&self) -> Result<(), LedgerClosingError> {
        todo!()
    }

    async fn execute_annual_closing_transaction(&self) -> Result<(), LedgerClosingError> {
        todo!()
    }
}
