use async_graphql::*;

use std::sync::Arc;

use lana_app::balance_sheet::BalanceSheet as DomainBalanceSheet;

use super::{
    LedgerAccountBalanceRange,
    ledger_account::{CHART_REF, LedgerAccount},
};

use lana_app::accounting::ledger_account::LedgerAccount as DomainLedgerAccount;

use admin_graphql_shared::primitives::LedgerAccountId;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct BalanceSheet {
    name: String,

    #[graphql(skip)]
    entity: Arc<DomainBalanceSheet>,
}

impl From<DomainBalanceSheet> for BalanceSheet {
    fn from(balance_sheet: DomainBalanceSheet) -> Self {
        BalanceSheet {
            name: balance_sheet.name.to_string(),
            entity: Arc::new(balance_sheet),
        }
    }
}

#[ComplexObject]
impl BalanceSheet {
    async fn balance(&self) -> async_graphql::Result<LedgerAccountBalanceRange> {
        if let Some(balance) = self.entity.btc_balance_range.as_ref() {
            Ok(Some(balance).into())
        } else {
            Ok(self.entity.usd_balance_range.as_ref().into())
        }
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
