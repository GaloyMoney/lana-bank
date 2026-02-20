use async_graphql::*;

use std::sync::Arc;

use lana_app::profit_and_loss::ProfitAndLossStatement as DomainProfitAndLossStatement;

use super::{
    BtcLedgerAccountBalanceRange, LedgerAccountBalanceRangeByCurrency,
    UsdLedgerAccountBalanceRange,
    ledger_account::{CHART_REF, LedgerAccount},
};

use lana_app::accounting::ledger_account::LedgerAccount as DomainLedgerAccount;

use admin_graphql_shared::primitives::LedgerAccountId;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ProfitAndLossStatement {
    pub name: String,
    #[graphql(skip)]
    pub entity: Arc<DomainProfitAndLossStatement>,
}

impl From<DomainProfitAndLossStatement> for ProfitAndLossStatement {
    fn from(profit_and_loss: DomainProfitAndLossStatement) -> Self {
        ProfitAndLossStatement {
            name: profit_and_loss.name.to_string(),
            entity: Arc::new(profit_and_loss),
        }
    }
}

#[ComplexObject]
impl ProfitAndLossStatement {
    async fn total(&self) -> async_graphql::Result<LedgerAccountBalanceRangeByCurrency> {
        Ok(LedgerAccountBalanceRangeByCurrency {
            usd: self
                .entity
                .usd_balance_range
                .as_ref()
                .map(UsdLedgerAccountBalanceRange::from)
                .unwrap_or_default(),
            btc: self
                .entity
                .btc_balance_range
                .as_ref()
                .map(BtcLedgerAccountBalanceRange::from)
                .unwrap_or_default(),
        })
    }

    async fn categories(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<LedgerAccount>> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let categories: std::collections::HashMap<LedgerAccountId, DomainLedgerAccount> = app
            .accounting()
            .find_all_ledger_accounts(CHART_REF, &self.entity.category_ids)
            .await?;

        Ok(self
            .entity
            .category_ids
            .iter()
            .filter_map(|id| categories.get(id).cloned())
            .map(LedgerAccount::from)
            .collect())
    }
}
