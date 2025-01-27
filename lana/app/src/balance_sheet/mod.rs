pub mod error;
pub mod ledger;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
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

    pub async fn create_balance_sheet(
        &self,
        id: impl Into<LedgerAccountSetId>,
        name: String,
    ) -> Result<BalanceSheetIds, BalanceSheetError> {
        let account_set_id: LedgerAccountSetId = id.into();

        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(op.tx(), Object::BalanceSheet, BalanceSheetAction::Create)
            .await?;

        Ok(self
            .balance_sheet_ledger
            .create(op, account_set_id, &name)
            .await?)
    }

    pub async fn find_by_name(
        &self,
        name: String,
    ) -> Result<Option<BalanceSheetIds>, BalanceSheetError> {
        self.authz
            .audit()
            .record_system_entry(Object::BalanceSheet, BalanceSheetAction::Read)
            .await?;

        let balance_sheets = self
            .balance_sheet_ledger
            .list_for_name(name.to_string(), Default::default())
            .await?
            .entities;

        let statement_id = match balance_sheets.len() {
            0 => return Ok(None),
            1 => balance_sheets[0].id,
            _ => return Err(BalanceSheetError::MultipleFound(name)),
        };

        let statement_members = self
            .balance_sheet_ledger
            .get_member_account_sets(statement_id)
            .await?;

        let assets_id = statement_members
            .iter()
            .find(|m| m.name == ASSETS_NAME)
            .ok_or(BalanceSheetError::NotFound(ASSETS_NAME.to_string()))?
            .id;

        let liabilities_id = statement_members
            .iter()
            .find(|m| m.name == LIABILITIES_NAME)
            .ok_or(BalanceSheetError::NotFound(LIABILITIES_NAME.to_string()))?
            .id;

        let equity_id = statement_members
            .iter()
            .find(|m| m.name == EQUITY_NAME)
            .ok_or(BalanceSheetError::NotFound(EQUITY_NAME.to_string()))?
            .id;

        let equity_members = self
            .balance_sheet_ledger
            .get_member_account_sets(equity_id)
            .await?;

        let net_income_id = equity_members
            .iter()
            .find(|m| m.name == NET_INCOME_NAME)
            .ok_or(BalanceSheetError::NotFound(NET_INCOME_NAME.to_string()))?
            .id;

        let net_income_members = self
            .balance_sheet_ledger
            .get_member_account_sets(net_income_id)
            .await?;

        let revenue_id = net_income_members
            .iter()
            .find(|m| m.name == NI_REVENUE_NAME)
            .ok_or(BalanceSheetError::NotFound(NI_REVENUE_NAME.to_string()))?
            .id;

        let expenses_id = net_income_members
            .iter()
            .find(|m| m.name == NI_EXPENSES_NAME)
            .ok_or(BalanceSheetError::NotFound(NI_EXPENSES_NAME.to_string()))?
            .id;

        Ok(Some(BalanceSheetIds {
            id: statement_id,
            assets: assets_id,
            liabilities: liabilities_id,
            equity: equity_id,
            revenue: revenue_id,
            expenses: expenses_id,
        }))
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
        statement_ids: BalanceSheetIds,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        self.add_to(statement_ids.assets, member_id).await
    }

    pub async fn add_to_liabilities(
        &self,
        statement_ids: BalanceSheetIds,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        self.add_to(statement_ids.liabilities, member_id).await
    }

    pub async fn add_to_equity(
        &self,
        statement_ids: BalanceSheetIds,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        self.add_to(statement_ids.equity, member_id).await
    }

    pub async fn add_to_revenue(
        &self,
        statement_ids: BalanceSheetIds,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        self.add_to(statement_ids.revenue, member_id).await
    }

    pub async fn add_to_expenses(
        &self,
        statement_ids: BalanceSheetIds,
        member_id: impl Into<LedgerAccountSetId>,
    ) -> Result<(), BalanceSheetError> {
        self.add_to(statement_ids.expenses, member_id).await
    }

    pub async fn balance_sheet(
        &self,
        sub: &Subject,
        name: String,
    ) -> Result<BalanceSheet, BalanceSheetError> {
        self.authz
            .enforce_permission(sub, Object::BalanceSheet, BalanceSheetAction::Read)
            .await?;

        let balance_sheet_ids = self
            .find_by_name(name.to_string())
            .await?
            .ok_or(BalanceSheetError::NotFound(name))?;

        Ok(self
            .balance_sheet_ledger
            .get_balance_sheet(balance_sheet_ids)
            .await?)
    }
}

#[derive(Clone)]
pub struct BalanceSheet {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub btc_balance: BtcStatementAccountSetBalance,
    pub usd_balance: UsdStatementAccountSetBalance,
    pub categories: Vec<StatementAccountSetWithAccounts>,
}
