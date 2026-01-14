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

/// Resolved account set IDs from the Chart of Accounts for linking
pub(crate) struct ChartAccountSetIds {
    pub assets: CalaAccountSetId,
    pub liabilities: CalaAccountSetId,
    pub equity: CalaAccountSetId,
    pub revenue: CalaAccountSetId,
    pub cost_of_revenue: CalaAccountSetId,
    pub expenses: CalaAccountSetId,
}

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
        skip(self, chart)
    )]
    pub async fn get_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
        chart: &Chart,
    ) -> Result<Option<AccountingBaseConfig>, BalanceSheetError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_balance_sheet_configuration(),
                CoreAccountingAction::BALANCE_SHEET_CONFIGURATION_READ,
            )
            .await?;

        let is_configured = self
            .balance_sheet_ledger
            .is_configured(reference)
            .await?;

        if is_configured {
            Ok(chart.find_accounting_base_config())
        } else {
            Ok(None)
        }
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
        // Check if already configured via Cala state
        let is_configured = self
            .balance_sheet_ledger
            .is_configured(reference.clone())
            .await?;
        if is_configured {
            return Err(BalanceSheetError::BalanceSheetConfigAlreadyExists);
        }

        let config = chart
            .find_accounting_base_config()
            .ok_or(BalanceSheetError::AccountingBaseConfigNotFound)?;

        // Resolve account codes to Cala account set IDs
        let chart_account_set_ids = ChartAccountSetIds {
            assets: chart.account_set_id_from_code(&config.assets_code)?,
            liabilities: chart.account_set_id_from_code(&config.liabilities_code)?,
            equity: chart.account_set_id_from_code(&config.equity_code)?,
            revenue: chart.account_set_id_from_code(&config.revenue_code)?,
            cost_of_revenue: chart.account_set_id_from_code(&config.cost_of_revenue_code)?,
            expenses: chart.account_set_id_from_code(&config.expenses_code)?,
        };

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_balance_sheet_configuration(),
                CoreAccountingAction::BALANCE_SHEET_CONFIGURATION_UPDATE,
            )
            .await?;

        // Attach chart account sets as members (only side effect)
        self.balance_sheet_ledger
            .attach_chart_of_accounts_account_sets(reference, chart_account_set_ids)
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
