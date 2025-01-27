pub mod error;
pub mod ledger;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use rbac_types::{ProfitAndLossStatementAction, Subject};

use crate::{
    authorization::{Authorization, Object},
    primitives::{LedgerAccountSetId, ProfitAndLossStatementId},
    statement::*,
};

use error::*;
use ledger::*;

pub(crate) const REVENUE_NAME: &str = "Revenue";
pub(crate) const EXPENSES_NAME: &str = "Expenses";

#[derive(Clone, Copy)]
pub struct ProfitAndLossStatementIds {
    pub id: ProfitAndLossStatementId,
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

    pub async fn create_pl_statement(
        &self,
        id: impl Into<ProfitAndLossStatementId>,
        name: String,
    ) -> Result<ProfitAndLossStatementIds, ProfitAndLossStatementError> {
        let account_set_id: LedgerAccountSetId = id.into().into();

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                Object::ProfitAndLossStatement,
                ProfitAndLossStatementAction::Create,
            )
            .await?;

        Ok(self
            .pl_statement_ledger
            .create(op, account_set_id, &name)
            .await?)
    }

    pub async fn find_by_name(
        &self,
        name: String,
    ) -> Result<Option<ProfitAndLossStatementIds>, ProfitAndLossStatementError> {
        self.authz
            .audit()
            .record_system_entry(
                Object::ProfitAndLossStatement,
                ProfitAndLossStatementAction::Read,
            )
            .await?;

        let pl_statements = self
            .pl_statement_ledger
            .list_for_name(name.to_string(), Default::default())
            .await?
            .entities;

        let statement_id = match pl_statements.len() {
            0 => return Ok(None),
            1 => pl_statements[0].id,
            _ => return Err(ProfitAndLossStatementError::MultipleFound(name)),
        };

        let members = self
            .pl_statement_ledger
            .get_member_account_sets(statement_id)
            .await?;

        let revenue_id = members
            .iter()
            .find(|m| m.name == REVENUE_NAME)
            .ok_or(ProfitAndLossStatementError::NotFound(
                REVENUE_NAME.to_string(),
            ))?
            .id;

        let expenses_id = members
            .iter()
            .find(|m| m.name == EXPENSES_NAME)
            .ok_or(ProfitAndLossStatementError::NotFound(
                EXPENSES_NAME.to_string(),
            ))?
            .id;

        Ok(Some(ProfitAndLossStatementIds {
            id: statement_id.into(),
            revenue: revenue_id,
            expenses: expenses_id,
        }))
    }

    pub async fn add_to_revenue(
        &self,
        statement_ids: ProfitAndLossStatementIds,
        member_id: impl Into<LedgerAccountSetId>,
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

        self.pl_statement_ledger
            .add_member(op, statement_ids.revenue, member_id)
            .await?;

        Ok(())
    }

    pub async fn add_to_expenses(
        &self,
        statement_ids: ProfitAndLossStatementIds,
        member_id: impl Into<LedgerAccountSetId>,
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

        let pl_statement_ids = self
            .find_by_name(name.to_string())
            .await?
            .ok_or(ProfitAndLossStatementError::NotFound(name))?;

        Ok(self
            .pl_statement_ledger
            .get_pl_statement(pl_statement_ids)
            .await?)
    }
}

#[derive(Clone)]
pub struct ProfitAndLossStatement {
    pub id: ProfitAndLossStatementId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalance,
    pub usd_balance: UsdStatementAccountSetBalance,
    pub categories: Vec<StatementAccountSetWithAccounts>,
}
