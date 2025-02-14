pub mod error;
pub mod ledger;

use chrono::{DateTime, Utc};

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use rbac_types::{CashFlowStatementAction, Subject};

use crate::{
    authorization::{Authorization, Object},
    primitives::LedgerAccountSetId,
    statement::*,
};

use error::*;
use ledger::*;

pub(crate) const FROM_OPERATIONS_NAME: &str = "Cash Flow From Operations";
pub(crate) const FROM_INVESTING_NAME: &str = "Cash Flow From Investing";
pub(crate) const FROM_FINANCING_NAME: &str = "Cash Flow From Financing";
pub(crate) const NET_INCOME_NAME: &str = "Net Income";
pub(crate) const REVENUE_NAME: &str = "Revenue";
pub(crate) const EXPENSES_NAME: &str = "Expenses";
pub(crate) const NON_CASH_ADJUSTMENTS_NAME: &str = "Adjustments for Non-Cash Items";
pub(crate) const FEE_INCOME_ADJUSTMENTS_NAME: &str = "Non-Cash Fee Income Adjustment";

#[derive(Clone, Copy)]
pub struct CashFlowStatementIds {
    pub id: LedgerAccountSetId,
    from_operations: LedgerAccountSetId,
    from_investing: LedgerAccountSetId,
    from_financing: LedgerAccountSetId,
    revenue: LedgerAccountSetId,
    expenses: LedgerAccountSetId,
    fee_income_adjustments: LedgerAccountSetId,
}

#[derive(Clone)]
pub struct CashFlowStatements {
    pool: sqlx::PgPool,
    authz: Authorization,
    cash_flow_statement_ledger: CashFlowStatementLedger,
}

impl CashFlowStatements {
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
    ) -> Result<Self, CashFlowStatementError> {
        let cash_flow_statement_ledger = CashFlowStatementLedger::new(cala, journal_id);

        Ok(Self {
            pool: pool.clone(),
            cash_flow_statement_ledger,
            authz: authz.clone(),
        })
    }

    pub async fn create_cash_flow_statement(
        &self,
        name: String,
    ) -> Result<(), CashFlowStatementError> {
        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                Object::CashFlowStatement,
                CashFlowStatementAction::Create,
            )
            .await?;

        match self.cash_flow_statement_ledger.create(op, &name).await {
            Ok(_) => Ok(()),
            Err(e) if e.account_set_exists() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn add_to(
        &self,
        account_set_id: LedgerAccountSetId,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), CashFlowStatementError> {
        let member_id = member_id.into();

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op.tx(),
                Object::CashFlowStatement,
                CashFlowStatementAction::Update,
            )
            .await?;

        self.cash_flow_statement_ledger
            .add_member(op, account_set_id, member_id)
            .await?;

        Ok(())
    }

    pub async fn add_to_from_operations(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), CashFlowStatementError> {
        let statement_ids = self
            .cash_flow_statement_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.from_operations, member_id).await
    }

    pub async fn add_to_from_investing(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), CashFlowStatementError> {
        let statement_ids = self
            .cash_flow_statement_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.from_investing, member_id).await
    }

    pub async fn add_to_from_financing(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), CashFlowStatementError> {
        let statement_ids = self
            .cash_flow_statement_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.from_financing, member_id).await
    }

    pub async fn add_to_revenue(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), CashFlowStatementError> {
        let statement_ids = self
            .cash_flow_statement_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.revenue, member_id).await
    }

    pub async fn add_to_expenses(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), CashFlowStatementError> {
        let statement_ids = self
            .cash_flow_statement_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.expenses, member_id).await
    }

    pub async fn add_to_fee_income_adjustments(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), CashFlowStatementError> {
        let statement_ids = self
            .cash_flow_statement_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.fee_income_adjustments, member_id)
            .await
    }

    pub async fn cash_flow_statement(
        &self,
        sub: &Subject,
        reference: String,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Result<CashFlowStatement, CashFlowStatementError> {
        self.authz
            .enforce_permission(
                sub,
                Object::CashFlowStatement,
                CashFlowStatementAction::Read,
            )
            .await?;

        Ok(self
            .cash_flow_statement_ledger
            .get_cash_flow_statement(reference, from, until)
            .await?)
    }
}

#[derive(Clone)]
pub struct CashFlowStatement {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalanceRange,
    pub usd_balance: UsdStatementAccountSetBalanceRange,
    pub categories: Vec<StatementAccountSetWithAccounts>,
}
