mod chart_of_accounts_integration;
pub mod error;
pub mod ledger;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use chrono::NaiveDate;
use tracing_macros::record_error_severity;

use crate::{
    AccountingBaseConfig, LedgerAccountId,
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

    #[record_error_severity]
    #[instrument(name = "core_accounting.balance_sheet.create", skip(self, name), fields(balance_sheet_name = %name))]
    pub async fn create_balance_sheet(&self, name: String) -> Result<(), BalanceSheetError> {
        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                &mut op,
                CoreAccountingObject::all_balance_sheet(),
                CoreAccountingAction::BALANCE_SHEET_CREATE,
            )
            .await?;

        match self.balance_sheet_ledger.create(&mut op, &name).await {
            Ok(_) => {
                op.commit().await?;
                Ok(())
            }
            Err(e) if e.account_set_exists() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.balance_sheet.get_integration_config",
        skip(self)
    )]
    pub async fn get_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
    ) -> Result<Option<AccountingBaseConfig>, BalanceSheetError> {
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

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.balance_sheet.set_integration_config",
        skip(self, chart)
    )]
    pub async fn set_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
        chart: &Chart,
    ) -> Result<AccountingBaseConfig, BalanceSheetError> {
        if self
            .balance_sheet_ledger
            .get_chart_of_accounts_integration_config(reference.to_string())
            .await?
            .is_some()
        {
            return Err(BalanceSheetError::BalanceSheetConfigAlreadyExists);
        }

        let config = match chart.find_accounting_base_config() {
            Some(config) => config,
            None => return Err(BalanceSheetError::AccountingBaseConfigNotFound),
        };

        let assets_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.assets_code)?;
        let liabilities_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.liabilities_code)?;
        let equity_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.equity_code)?;
        let revenue_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.revenue_code)?;
        let cost_of_revenue_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.cost_of_revenue_code)?;
        let expenses_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.expenses_code)?;

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

    #[record_error_severity]
    #[instrument(name = "core_accounting.balance_sheet.balance_sheet", skip(self))]
    pub async fn balance_sheet(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
        from: NaiveDate,
        until: Option<NaiveDate>,
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
