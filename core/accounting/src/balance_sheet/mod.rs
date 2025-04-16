mod chart_of_accounts_integration;
pub mod error;
pub mod ledger;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use chrono::{DateTime, Utc};

use crate::{
    LedgerAccountId,
    chart_of_accounts::Chart,
    primitives::{BalanceRange, CalaAccountSetId, CoreAccountingAction, CoreAccountingObject},
};

pub use chart_of_accounts_integration::ChartOfAccountsIntegrationConfig;
use error::*;
use ledger::*;

pub(crate) const ASSETS_NAME: &str = "Assets";
pub(crate) const LIABILITIES_NAME: &str = "Liabilities";
pub(crate) const EQUITY_NAME: &str = "Equity";
pub(crate) const NET_INCOME_NAME: &str = "Current Earnings";
pub(crate) const REVENUE_NAME: &str = "Revenue";
pub(crate) const COST_OF_REVENUE_NAME: &str = "Cost of Revenue";
pub(crate) const EXPENSES_NAME: &str = "Expenses";

#[derive(Clone, Copy)]
pub struct BalanceSheetIds {
    pub id: CalaAccountSetId,
    pub assets: CalaAccountSetId,
    pub liabilities: CalaAccountSetId,
    pub equity: CalaAccountSetId,
    pub revenue: CalaAccountSetId,
    pub cost_of_revenue: CalaAccountSetId,
    pub expenses: CalaAccountSetId,
}

impl BalanceSheetIds {
    fn internal_ids(&self) -> Vec<CalaAccountSetId> {
        let Self {
            id: _id,

            assets,
            liabilities,
            equity,
            revenue,
            cost_of_revenue,
            expenses,
        } = self;

        vec![
            *assets,
            *liabilities,
            *equity,
            *revenue,
            *cost_of_revenue,
            *expenses,
        ]
    }

    fn account_set_id_for_config(&self) -> CalaAccountSetId {
        self.revenue
    }
}

#[derive(Clone)]
pub struct BalanceSheets<Perms>
where
    Perms: PermissionCheck,
{
    pool: sqlx::PgPool,
    authz: Perms,
    balance_sheet_ledger: BalanceSheetLedger,
}

impl<Perms> BalanceSheets<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
    ) -> Self {
        let balance_sheet_ledger = BalanceSheetLedger::new(cala, journal_id);

        Self {
            pool: pool.clone(),
            balance_sheet_ledger,
            authz: authz.clone(),
        }
    }

    pub async fn create_balance_sheet(&self, name: String) -> Result<(), BalanceSheetError> {
        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                CoreAccountingObject::all_balance_sheet(),
                CoreAccountingAction::BALANCE_SHEET_CREATE,
            )
            .await?;

        match self.balance_sheet_ledger.create(op, &name).await {
            Ok(_) => Ok(()),
            Err(e) if e.account_set_exists() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, BalanceSheetError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_balance_sheet_configuration(),
                CoreAccountingAction::BALANCE_SHEET_CONFIGURATION_READ,
            )
            .await?;
        Ok(self
            .balance_sheet_ledger
            .get_chart_of_accounts_integration_config(reference)
            .await?)
    }

    pub async fn set_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
        chart: &Chart,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, BalanceSheetError> {
        if chart.id != config.chart_of_accounts_id {
            return Err(BalanceSheetError::ChartIdMismatch);
        }

        if self
            .balance_sheet_ledger
            .get_chart_of_accounts_integration_config(reference.to_string())
            .await?
            .is_some()
        {
            return Err(BalanceSheetError::CreditConfigAlreadyExists);
        }

        let assets_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.chart_of_accounts_assets_code)?;
        let liabilities_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.chart_of_accounts_liabilities_code)?;
        let equity_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.chart_of_accounts_equity_code)?;
        let revenue_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.chart_of_accounts_revenue_code)?;
        let cost_of_revenue_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.chart_of_accounts_cost_of_revenue_code)?;
        let expenses_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.chart_of_accounts_expenses_code)?;

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_balance_sheet_configuration(),
                CoreAccountingAction::BALANCE_SHEET_CONFIGURATION_UPDATE,
            )
            .await?;

        let charts_integration_meta = ChartOfAccountsIntegrationMeta {
            audit_info,
            config: config.clone(),

            assets_child_account_set_id_from_chart,
            liabilities_child_account_set_id_from_chart,
            equity_child_account_set_id_from_chart,
            revenue_child_account_set_id_from_chart,
            cost_of_revenue_child_account_set_id_from_chart,
            expenses_child_account_set_id_from_chart,
        };

        self.balance_sheet_ledger
            .attach_chart_of_accounts_account_sets(reference, charts_integration_meta)
            .await?;

        Ok(config)
    }

    pub async fn balance_sheet(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<BalanceSheet, BalanceSheetError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_balance_sheet(),
                CoreAccountingAction::BALANCE_SHEET_READ,
            )
            .await?;

        Ok(self
            .balance_sheet_ledger
            .get_balance_sheet(reference, from, until)
            .await?)
    }
}

#[derive(Clone)]
pub struct BalanceSheet {
    pub id: LedgerAccountId,
    pub name: String,
    pub usd_balance_range: Option<BalanceRange>,
    pub btc_balance_range: Option<BalanceRange>,
    pub category_ids: Vec<LedgerAccountId>,
}
