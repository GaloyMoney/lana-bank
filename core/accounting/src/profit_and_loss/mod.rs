mod chart_of_accounts_integration;
pub mod error;
pub mod ledger;

use chrono::NaiveDate;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use tracing_macros::record_error_severity;

use crate::{
    AccountingBaseConfig, LedgerAccountId,
    chart_of_accounts::Chart,
    primitives::{BalanceRange, CalaAccountSetId, CoreAccountingAction, CoreAccountingObject},
};

pub use chart_of_accounts_integration::ChartOfAccountsIntegrationConfig;
use error::*;
use ledger::*;

pub(crate) const REVENUE_NAME: &str = "Revenue";
pub(crate) const EXPENSES_NAME: &str = "Expenses";
pub(crate) const COST_OF_REVENUE_NAME: &str = "Cost of Revenue";
#[derive(Clone, Copy)]
pub struct ProfitAndLossStatementIds {
    pub id: CalaAccountSetId,
    pub revenue: CalaAccountSetId,
    pub cost_of_revenue: CalaAccountSetId,
    pub expenses: CalaAccountSetId,
}

impl ProfitAndLossStatementIds {
    fn internal_ids(&self) -> Vec<CalaAccountSetId> {
        let Self {
            id: _id,
            revenue,
            cost_of_revenue,
            expenses,
        } = self;

        vec![*revenue, *cost_of_revenue, *expenses]
    }

    fn account_set_id_for_config(&self) -> CalaAccountSetId {
        self.revenue
    }
}

#[derive(Clone)]
pub struct ProfitAndLossStatements<Perms>
where
    Perms: PermissionCheck,
{
    pool: sqlx::PgPool,
    authz: Perms,
    pl_statement_ledger: ProfitAndLossStatementLedger,
}

impl<Perms> ProfitAndLossStatements<Perms>
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
        let pl_statement_ledger = ProfitAndLossStatementLedger::new(cala, journal_id);

        Self {
            pool: pool.clone(),
            pl_statement_ledger,
            authz: authz.clone(),
        }
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.profit_and_loss.create_pl_statement", skip(self, name), fields(pl_statement_name = %name))]
    pub async fn create_pl_statement(
        &self,
        name: String,
    ) -> Result<(), ProfitAndLossStatementError> {
        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                &mut op,
                CoreAccountingObject::all_profit_and_loss(),
                CoreAccountingAction::PROFIT_AND_LOSS_CREATE,
            )
            .await?;

        match self.pl_statement_ledger.create(&mut op, &name).await {
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
        name = "core_accounting.profit_and_loss.get_chart_of_accounts_integration_config",
        skip(self, chart)
    )]
    pub async fn get_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
        chart: &Chart,
    ) -> Result<Option<AccountingBaseConfig>, ProfitAndLossStatementError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_profit_and_loss_configuration(),
                CoreAccountingAction::PROFIT_AND_LOSS_CONFIGURATION_READ,
            )
            .await?;

        let is_configured = self
            .pl_statement_ledger
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
        name = "core_accounting.profit_and_loss.set_chart_of_accounts_integration_config",
        skip(self, chart)
    )]
    pub async fn set_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
        chart: &Chart,
    ) -> Result<AccountingBaseConfig, ProfitAndLossStatementError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_profit_and_loss_configuration(),
                CoreAccountingAction::PROFIT_AND_LOSS_CONFIGURATION_UPDATE,
            )
            .await?;

        // Check if already configured via Cala state
        let is_configured = self
            .pl_statement_ledger
            .is_configured(reference.clone())
            .await?;
        if is_configured {
            return Err(ProfitAndLossStatementError::ProfitAndLossStatementConfigAlreadyExists);
        }

        let config = chart
            .find_accounting_base_config()
            .ok_or(ProfitAndLossStatementError::AccountingBaseConfigNotFound)?;

        let revenue_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.revenue_code)?;
        let cost_of_revenue_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.cost_of_revenue_code)?;
        let expenses_child_account_set_id_from_chart =
            chart.account_set_id_from_code(&config.expenses_code)?;

        let charts_integration_meta = ChartOfAccountsIntegrationMeta {
            audit_info,
            config: config.clone(),

            revenue_child_account_set_id_from_chart,
            cost_of_revenue_child_account_set_id_from_chart,
            expenses_child_account_set_id_from_chart,
        };

        self.pl_statement_ledger
            .attach_chart_of_accounts_account_sets(reference, charts_integration_meta)
            .await?;

        Ok(config)
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.profit_and_loss.pl_statement", skip(self))]
    pub async fn pl_statement(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<ProfitAndLossStatement, ProfitAndLossStatementError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_profit_and_loss(),
                CoreAccountingAction::PROFIT_AND_LOSS_READ,
            )
            .await?;

        Ok(self
            .pl_statement_ledger
            .get_pl_statement(reference, from, until)
            .await?)
    }
}

#[derive(Clone)]
pub struct ProfitAndLossStatement {
    pub id: LedgerAccountId,
    pub name: String,
    pub usd_balance_range: Option<BalanceRange>,
    pub btc_balance_range: Option<BalanceRange>,
    pub category_ids: Vec<LedgerAccountId>,
}
