use async_graphql::*;
use chrono::NaiveDate;

use lana_app::balance_sheet::{
    BalanceSheet as DomainBalanceSheet, BalanceSheetAccountSet as DomainBalanceSheetAccountSet,
};

use super::{AccountCode, LedgerAccountBalanceByCurrency};
use crate::{
    graphql::loader::{BalanceSheetAccountSetKey, LanaDataLoader},
    primitives::*,
};

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct BalanceSheet {
    name: String,

    #[graphql(skip)]
    entity: Arc<DomainBalanceSheet>,
    #[graphql(skip)]
    as_of: NaiveDate,
}

impl BalanceSheet {
    pub fn new(balance_sheet: DomainBalanceSheet, as_of: NaiveDate) -> Self {
        Self {
            name: balance_sheet.name.to_string(),
            entity: Arc::new(balance_sheet),
            as_of,
        }
    }
}

#[ComplexObject]
impl BalanceSheet {
    async fn assets_balance(&self) -> async_graphql::Result<LedgerAccountBalanceByCurrency> {
        Ok((&self.entity.assets).into())
    }

    async fn liabilities_balance(&self) -> async_graphql::Result<LedgerAccountBalanceByCurrency> {
        Ok((&self.entity.liabilities).into())
    }

    async fn equity_balance(&self) -> async_graphql::Result<LedgerAccountBalanceByCurrency> {
        Ok((&self.entity.equity).into())
    }

    async fn categories(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<BalanceSheetAccount>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let keys = self
            .entity
            .category_ids
            .iter()
            .copied()
            .map(|id| BalanceSheetAccountSetKey {
                id,
                as_of: self.as_of,
            })
            .collect::<Vec<_>>();
        let categories = loader.load_many(keys.clone()).await?;

        Ok(keys
            .into_iter()
            .filter_map(|key| categories.get(&key).cloned())
            .collect())
    }
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct BalanceSheetAccountSet {
    balance_sheet_account_id: ID,
    ledger_account_id: UUID,
    code: Option<AccountCode>,
    name: String,

    #[graphql(skip)]
    entity: Arc<DomainBalanceSheetAccountSet>,
    #[graphql(skip)]
    as_of: NaiveDate,
}

impl BalanceSheetAccount {
    pub fn new(account: DomainBalanceSheetAccountSet, as_of: NaiveDate) -> Self {
        Self {
            balance_sheet_account_id: account.id.to_global_id(),
            ledger_account_id: UUID::from(account.id),
            code: account.code.as_ref().map(|code| code.into()),
            name: account.name.clone(),
            entity: Arc::new(account),
            as_of,
        }
    }
}

#[ComplexObject]
impl BalanceSheetAccount {
    async fn balance(&self) -> async_graphql::Result<LedgerAccountBalanceByCurrency> {
        Ok((&self.entity.balance).into())
    }

    async fn children(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<BalanceSheetAccount>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let keys = self
            .entity
            .children_ids
            .iter()
            .copied()
            .map(|id| BalanceSheetAccountSetKey {
                id,
                as_of: self.as_of,
            })
            .collect::<Vec<_>>();
        let children = loader.load_many(keys.clone()).await?;

        Ok(keys
            .into_iter()
            .filter_map(|key| children.get(&key).cloned())
            .collect())
    }
}
