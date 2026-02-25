use async_graphql::*;

use admin_graphql_accounting::LedgerAccount;

use crate::primitives::*;

pub const CHART_REF: &str = admin_graphql_accounting::CHART_REF;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CreditFacilityLedgerAccounts {
    pub facility_account_id: UUID,
    pub disbursed_receivable_not_yet_due_account_id: UUID,
    pub disbursed_receivable_due_account_id: UUID,
    pub disbursed_receivable_overdue_account_id: UUID,
    pub disbursed_defaulted_account_id: UUID,
    pub collateral_account_id: UUID,
    pub collateral_in_liquidation_account_id: UUID,
    pub liquidated_collateral_account_id: UUID,
    pub proceeds_from_liquidation_account_id: UUID,
    pub interest_receivable_not_yet_due_account_id: UUID,
    pub interest_receivable_due_account_id: UUID,
    pub interest_receivable_overdue_account_id: UUID,
    pub interest_defaulted_account_id: UUID,
    pub interest_income_account_id: UUID,
    pub fee_income_account_id: UUID,
    pub payment_holding_account_id: UUID,
    pub uncovered_outstanding_account_id: UUID,
}

macro_rules! load_ledger_account {
    ($self:expr, $ctx:expr, $field:ident) => {{
        let (app, _sub) = app_and_sub_from_ctx!($ctx);
        let accounts: std::collections::HashMap<_, LedgerAccount> = app
            .accounting()
            .find_all_ledger_accounts(CHART_REF, &[LedgerAccountId::from($self.$field)])
            .await?;
        Ok(accounts
            .into_values()
            .next()
            .expect("Ledger account not found"))
    }};
}

#[ComplexObject]
impl CreditFacilityLedgerAccounts {
    async fn facility_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, facility_account_id)
    }
    async fn disbursed_receivable_not_yet_due_account(
        &self,
        ctx: &Context<'_>,
    ) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, disbursed_receivable_not_yet_due_account_id)
    }
    async fn disbursed_receivable_due_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, disbursed_receivable_due_account_id)
    }
    async fn disbursed_receivable_overdue_account(
        &self,
        ctx: &Context<'_>,
    ) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, disbursed_receivable_overdue_account_id)
    }
    async fn disbursed_defaulted_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, disbursed_defaulted_account_id)
    }
    async fn collateral_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, collateral_account_id)
    }
    async fn collateral_in_liquidation_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, collateral_in_liquidation_account_id)
    }
    async fn liquidated_collateral_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, liquidated_collateral_account_id)
    }
    async fn proceeds_from_liquidation_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, proceeds_from_liquidation_account_id)
    }
    async fn interest_receivable_not_yet_due_account(
        &self,
        ctx: &Context<'_>,
    ) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, interest_receivable_not_yet_due_account_id)
    }
    async fn interest_receivable_due_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, interest_receivable_due_account_id)
    }
    async fn interest_receivable_overdue_account(
        &self,
        ctx: &Context<'_>,
    ) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, interest_receivable_overdue_account_id)
    }
    async fn interest_defaulted_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, interest_defaulted_account_id)
    }
    async fn interest_income_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, interest_income_account_id)
    }
    async fn fee_income_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, fee_income_account_id)
    }
    async fn payment_holding_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, payment_holding_account_id)
    }
    async fn uncovered_outstanding_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, uncovered_outstanding_account_id)
    }
}
