pub mod error;
pub mod ledger;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use chrono::{DateTime, Utc};
use rbac_types::{BalanceSheetAction, Subject};

use crate::{
    authorization::{Authorization, Object},
    primitives::LedgerAccountSetId,
    statement::*,
};

use error::*;
use ledger::*;

pub(crate) const ASSETS_NAME: &str = "Assets";
pub(crate) const LIABILITIES_NAME: &str = "Liabilities";
pub(crate) const EQUITY_NAME: &str = "Equity";
pub(crate) const NET_INCOME_NAME: &str = "Net Income";
pub(crate) const NI_REVENUE_NAME: &str = "Revenue";
pub(crate) const NI_EXPENSES_NAME: &str = "Expenses";

#[derive(Clone, Copy)]
pub struct BalanceSheetIds {
    pub id: LedgerAccountSetId,
    pub assets: LedgerAccountSetId,
    pub liabilities: LedgerAccountSetId,
    pub equity: LedgerAccountSetId,
    pub revenue: LedgerAccountSetId,
    pub expenses: LedgerAccountSetId,
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

    async fn add_to(
        &self,
        account_set_id: LedgerAccountSetId,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        let member_id = member_id.into();

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(op.tx(), Object::BalanceSheet, BalanceSheetAction::Update)
            .await?;

        self.balance_sheet_ledger
            .add_member(op, account_set_id, member_id)
            .await?;

        Ok(())
    }

    pub async fn add_to_assets(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        let statement_ids = self
            .balance_sheet_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.assets, member_id).await
    }

    pub async fn add_to_liabilities(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        let statement_ids = self
            .balance_sheet_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.liabilities, member_id).await
    }

    pub async fn add_to_equity(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        let statement_ids = self
            .balance_sheet_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.equity, member_id).await
    }

    pub async fn add_to_revenue(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        let statement_ids = self
            .balance_sheet_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.revenue, member_id).await
    }

    pub async fn add_to_expenses(
        &self,
        reference: String,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        let statement_ids = self
            .balance_sheet_ledger
            .get_ids_from_reference(reference)
            .await?;

        self.add_to(statement_ids.expenses, member_id).await
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
