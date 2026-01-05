use async_graphql::*;

use crate::{graphql::accounting::LedgerAccount, primitives::*};

use super::LanaDataLoader;

#[derive(SimpleObject)]
#[graphql(complex)]
pub(super) struct CreditFacilityLedgerAccounts {
    pub facility_account_id: UUID,
    pub disbursed_receivable_not_yet_due_account_id: UUID,
    pub disbursed_receivable_due_account_id: UUID,
    pub disbursed_receivable_overdue_account_id: UUID,
    pub disbursed_defaulted_account_id: UUID,
    pub collateral_account_id: UUID,
    pub collateral_in_liquidation_account_id: UUID,
    pub proceeds_from_liquidation_account_id: UUID,
    pub interest_receivable_not_yet_due_account_id: UUID,
    pub interest_receivable_due_account_id: UUID,
    pub interest_receivable_overdue_account_id: UUID,
    pub interest_defaulted_account_id: UUID,
    pub interest_income_account_id: UUID,
    pub fee_income_account_id: UUID,
}

#[ComplexObject]
impl CreditFacilityLedgerAccounts {
    async fn facility_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility_account = loader
            .load_one(LedgerAccountId::from(self.facility_account_id))
            .await?
            .expect("Ledger account not found");
        Ok(facility_account)
    }
    async fn disbursed_receivable_not_yet_due_account(
        &self,
        ctx: &Context<'_>,
    ) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let disbursed_receivable_not_yet_due_account = loader
            .load_one(LedgerAccountId::from(
                self.disbursed_receivable_not_yet_due_account_id,
            ))
            .await?
            .expect("Ledger account not found");
        Ok(disbursed_receivable_not_yet_due_account)
    }
    async fn disbursed_receivable_due_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let disbursed_receivable_due_account = loader
            .load_one(LedgerAccountId::from(
                self.disbursed_receivable_due_account_id,
            ))
            .await?
            .expect("Ledger account not found");
        Ok(disbursed_receivable_due_account)
    }
    async fn disbursed_receivable_overdue_account(
        &self,
        ctx: &Context<'_>,
    ) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let disbursed_receivable_overdue_account = loader
            .load_one(LedgerAccountId::from(
                self.disbursed_receivable_overdue_account_id,
            ))
            .await?
            .expect("Ledger account not found");
        Ok(disbursed_receivable_overdue_account)
    }
    async fn disbursed_defaulted_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let disbursed_defaulted_account = loader
            .load_one(LedgerAccountId::from(self.disbursed_defaulted_account_id))
            .await?
            .expect("Ledger account not found");
        Ok(disbursed_defaulted_account)
    }
    async fn collateral_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral_account = loader
            .load_one(LedgerAccountId::from(self.collateral_account_id))
            .await?
            .expect("Ledger account not found");
        Ok(collateral_account)
    }
    async fn collateral_in_liquidation_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral_in_liquidation_account = loader
            .load_one(LedgerAccountId::from(
                self.collateral_in_liquidation_account_id,
            ))
            .await?
            .expect("Ledger account not found");
        Ok(collateral_in_liquidation_account)
    }
    async fn proceeds_from_liquidation_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let proceeds_from_liquidation_account = loader
            .load_one(LedgerAccountId::from(
                self.proceeds_from_liquidation_account_id,
            ))
            .await?
            .expect("Ledger account not found");
        Ok(proceeds_from_liquidation_account)
    }
    async fn interest_receivable_not_yet_due_account(
        &self,
        ctx: &Context<'_>,
    ) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let interest_receivable_not_yet_due_account = loader
            .load_one(LedgerAccountId::from(
                self.interest_receivable_not_yet_due_account_id,
            ))
            .await?
            .expect("Ledger account not found");
        Ok(interest_receivable_not_yet_due_account)
    }
    async fn interest_receivable_due_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let interest_receivable_due_account = loader
            .load_one(LedgerAccountId::from(
                self.interest_receivable_due_account_id,
            ))
            .await?
            .expect("Ledger account not found");
        Ok(interest_receivable_due_account)
    }
    async fn interest_receivable_overdue_account(
        &self,
        ctx: &Context<'_>,
    ) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let interest_receivable_overdue_account = loader
            .load_one(LedgerAccountId::from(
                self.interest_receivable_overdue_account_id,
            ))
            .await?
            .expect("Ledger account not found");
        Ok(interest_receivable_overdue_account)
    }
    async fn interest_defaulted_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let interest_defaulted_account = loader
            .load_one(LedgerAccountId::from(self.interest_defaulted_account_id))
            .await?
            .expect("Ledger account not found");
        Ok(interest_defaulted_account)
    }
    async fn interest_income_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let interest_income_account = loader
            .load_one(LedgerAccountId::from(self.interest_income_account_id))
            .await?
            .expect("Ledger account not found");
        Ok(interest_income_account)
    }
    async fn fee_income_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let fee_income_account = loader
            .load_one(LedgerAccountId::from(self.fee_income_account_id))
            .await?
            .expect("Ledger account not found");
        Ok(fee_income_account)
    }
}
