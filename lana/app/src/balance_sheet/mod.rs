mod chart_of_accounts_integration;
pub mod error;
pub mod ledger;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use chrono::{DateTime, Utc};
use rbac_types::{BalanceSheetAction, BalanceSheetConfigurationAction, Subject};

use chart_of_accounts::Chart;

use crate::{
    authorization::{Authorization, Object},
    primitives::LedgerAccountSetId,
    statement::*,
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
    pub id: LedgerAccountSetId,
    pub assets: LedgerAccountSetId,
    pub liabilities: LedgerAccountSetId,
    pub equity: LedgerAccountSetId,
    pub revenue: LedgerAccountSetId,
    pub cost_of_revenue: LedgerAccountSetId,
    pub expenses: LedgerAccountSetId,
}

impl BalanceSheetIds {
    fn as_vec(&self) -> Vec<LedgerAccountSetId> {
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
}

#[derive(Clone)]
pub struct BalanceSheets {
    pool: sqlx::PgPool,
    authz: Authorization,
    balance_sheet_ledger: BalanceSheetLedger,
}

impl BalanceSheets {
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
    ) -> Result<Self, BalanceSheetError> {
        let balance_sheet_ledger = BalanceSheetLedger::new(cala, journal_id);

        Ok(Self {
            pool: pool.clone(),
            balance_sheet_ledger,
            authz: authz.clone(),
        })
    }

    pub async fn create_balance_sheet(&self, name: String) -> Result<(), BalanceSheetError> {
        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(op.tx(), Object::BalanceSheet, BalanceSheetAction::Create)
            .await?;

        match self.balance_sheet_ledger.create(op, &name).await {
            Ok(_) => Ok(()),
            Err(e) if e.account_set_exists() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn set_chart_of_accounts_integration_config(
        &self,
        sub: &Subject,
        reference: String,
        chart: Chart,
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
                Object::BalanceSheetConfiguration,
                BalanceSheetConfigurationAction::Update,
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
        sub: &Subject,
        reference: String,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<BalanceSheet, BalanceSheetError> {
        self.authz
            .enforce_permission(sub, Object::BalanceSheet, BalanceSheetAction::Read)
            .await?;

        Ok(self
            .balance_sheet_ledger
            .get_balance_sheet(reference, from, until)
            .await?)
    }
}

#[derive(Clone)]
pub struct BalanceSheet {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalanceRange,
    pub usd_balance: UsdStatementAccountSetBalanceRange,
    pub categories: Vec<StatementAccountSetWithAccounts>,
}
