pub mod error;
pub mod ledger;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use rbac_types::{ProfitAndLossStatementAction, Subject};

use crate::{
    authorization::{Authorization, Object},
    primitives::LedgerAccountSetId,
    statement::*,
};

use error::*;
use ledger::*;

pub(crate) const REVENUE_NAME: &str = "Revenue";
pub(crate) const EXPENSES_NAME: &str = "Expenses";

#[derive(Clone, Copy)]
pub struct ProfitAndLossStatementIds {
    pub id: LedgerAccountSetId,
    pub revenue: LedgerAccountSetId,
    pub expenses: LedgerAccountSetId,
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

    pub async fn find_or_create_pl_statement(
        &self,
        name: String,
    ) -> Result<ProfitAndLossStatementIds, ProfitAndLossStatementError> {
        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                Object::ProfitAndLossStatement,
                ProfitAndLossStatementAction::FindOrCreate,
            )
            .await?;

        Ok(self.pl_statement_ledger.find_or_create(op, &name).await?)
    }

    pub async fn add_to_revenue(
        &self,
        name: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), ProfitAndLossStatementError> {
        let member_id = member_id.into();
        let statement_ids = self
            .pl_statement_ledger
            .find_by_name(name.to_string())
            .await?;

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                Object::ProfitAndLossStatement,
                ProfitAndLossStatementAction::Update,
            )
            .await?;

        self.pl_statement_ledger
            .add_member(op, statement_ids.revenue, member_id)
            .await?;

        Ok(())
    }

    pub async fn add_to_expenses(
        &self,
        name: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), ProfitAndLossStatementError> {
        let member_id = member_id.into();
        let statement_ids = self
            .pl_statement_ledger
            .find_by_name(name.to_string())
            .await?;

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                Object::ProfitAndLossStatement,
                ProfitAndLossStatementAction::Update,
            )
            .await?;

        self.pl_statement_ledger
            .add_member(op, statement_ids.expenses, member_id)
            .await?;

        Ok(())
    }

    pub async fn pl_statement(
        &self,
        sub: &Subject,
        name: String,
    ) -> Result<ProfitAndLossStatement, ProfitAndLossStatementError> {
        self.authz
            .enforce_permission(
                sub,
                Object::ProfitAndLossStatement,
                ProfitAndLossStatementAction::Read,
            )
            .await?;

        Ok(self.pl_statement_ledger.get_pl_statement(name).await?)
    }
}

#[derive(Clone)]
pub struct ProfitAndLossStatement {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalance,
    pub usd_balance: UsdStatementAccountSetBalance,
    pub categories: Vec<StatementAccountSetWithAccounts>,
}
