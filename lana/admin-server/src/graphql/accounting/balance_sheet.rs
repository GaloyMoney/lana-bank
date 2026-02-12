use async_graphql::*;

use lana_app::balance_sheet::BalanceSheet as DomainBalanceSheet;

use crate::{graphql::loader::*, primitives::*};

use super::{LedgerAccount, LedgerAccountBalanceRange};

#[derive(SimpleObject)]
#[graphql(complex)]
pub(crate) struct BalanceSheet {
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
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let categories = loader
            .load_many(self.entity.category_ids.iter().copied())
            .await?;

        Ok(self
            .entity
            .category_ids
            .iter()
            .filter_map(|id| categories.get(id).cloned())
            .collect())
    }
}
