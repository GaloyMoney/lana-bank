mod chart_of_accounts_integration;
pub mod error;
pub mod ledger;

use chrono::{DateTime, Utc};

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use rbac_types::{
    ProfitAndLossStatementAction, ProfitAndLossStatementConfigurationAction, Subject,
};

use crate::{
    accounting::Chart,
    authorization::{Authorization, Object},
    primitives::CalaAccountSetId,
    statement::*,
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
pub struct ProfitAndLossStatements {
    pool: sqlx::PgPool,
    authz: Authorization,
    pl_statement_ledger: ProfitAndLossStatementLedger,
}

impl ProfitAndLossStatements {
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
    ) -> Result<Self, ProfitAndLossStatementError> {
        let pl_statement_ledger = ProfitAndLossStatementLedger::new(cala, journal_id);

        Ok(Self {
            pool: pool.clone(),
            pl_statement_ledger,
            authz: authz.clone(),
        })
    }

    pub async fn create_pl_statement(
        &self,
        name: String,
    ) -> Result<(), ProfitAndLossStatementError> {
        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                Object::ProfitAndLossStatement,
                ProfitAndLossStatementAction::Create,
            )
            .await?;

        match self.pl_statement_ledger.create(op, &name).await {
            Ok(_) => Ok(()),
            Err(e) if e.account_set_exists() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        sub: &Subject,
        reference: String,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, ProfitAndLossStatementError> {
        self.authz
            .enforce_permission(
                sub,
                Object::ProfitAndLossStatementConfiguration,
                ProfitAndLossStatementConfigurationAction::Read,
            )
            .await?;
        Ok(self
            .pl_statement_ledger
            .get_chart_of_accounts_integration_config(reference)
            .await?)
    }

    pub async fn set_chart_of_accounts_integration_config(
        &self,
        sub: &Subject,
        reference: String,
        chart: Chart,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, ProfitAndLossStatementError> {
        if chart.id != config.chart_of_accounts_id {
            return Err(ProfitAndLossStatementError::ChartIdMismatch);
        }

        if self
            .pl_statement_ledger
            .get_chart_of_accounts_integration_config(reference.to_string())
            .await?
            .is_some()
        {
            return Err(ProfitAndLossStatementError::ChartConfigAlreadyExists);
        }

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
                Object::ProfitAndLossStatementConfiguration,
                ProfitAndLossStatementConfigurationAction::Update,
            )
            .await?;

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

    pub async fn add_to_revenue(
        &self,
        reference: String,
        member_id: impl Into<CalaAccountSetId>,
    ) -> Result<(), ProfitAndLossStatementError> {
        let member_id = member_id.into();

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                Object::ProfitAndLossStatement,
                ProfitAndLossStatementAction::Update,
            )
            .await?;

        let statement_ids = self
            .pl_statement_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.pl_statement_ledger
            .add_member(op, statement_ids.revenue, member_id)
            .await?;

        Ok(())
    }

    pub async fn add_to_expenses(
        &self,
        reference: String,
        member_id: impl Into<CalaAccountSetId>,
    ) -> Result<(), ProfitAndLossStatementError> {
        let member_id = member_id.into();

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                Object::ProfitAndLossStatement,
                ProfitAndLossStatementAction::Update,
            )
            .await?;

        let statement_ids = self
            .pl_statement_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.pl_statement_ledger
            .add_member(op, statement_ids.expenses, member_id)
            .await?;

        Ok(())
    }

    pub async fn pl_statement(
        &self,
        sub: &Subject,
        reference: String,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<ProfitAndLossStatement, ProfitAndLossStatementError> {
        self.authz
            .enforce_permission(
                sub,
                Object::ProfitAndLossStatement,
                ProfitAndLossStatementAction::Read,
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
    pub id: CalaAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalanceRange,
    pub usd_balance: UsdStatementAccountSetBalanceRange,
    pub categories: Vec<StatementAccountSetWithAccounts>,
}
