use async_graphql::*;

use crate::accounting::LedgerAccount;

use super::primitives::*;

pub const CHART_REF: &str = crate::accounting::CHART_REF;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct DepositAccountLedgerAccounts {
    pub deposit_account_id: UUID,
    pub frozen_deposit_account_id: UUID,
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
impl DepositAccountLedgerAccounts {
    async fn deposit_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, deposit_account_id)
    }
    async fn frozen_deposit_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        load_ledger_account!(self, ctx, frozen_deposit_account_id)
    }
}
