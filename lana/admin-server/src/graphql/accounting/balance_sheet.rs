use async_graphql::*;
use chrono::NaiveDate;

use lana_app::{
    accounting::ledger_account::LedgerAccount as DomainLedgerAccount,
    balance_sheet::BalanceSheet as DomainBalanceSheet,
};

use super::{AccountCode, LedgerAccountBalanceRange};
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
    from: NaiveDate,
    #[graphql(skip)]
    until: Option<NaiveDate>,
}

impl BalanceSheet {
    pub fn new(
        balance_sheet: DomainBalanceSheet,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Self {
        Self {
            name: balance_sheet.name.to_string(),
            entity: Arc::new(balance_sheet),
            from,
            until,
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

    async fn categories(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<BalanceSheetAccountSet>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let keys = self
            .entity
            .category_ids
            .iter()
            .copied()
            .map(|id| BalanceSheetAccountSetKey {
                id,
                from: self.from,
                until: self.until,
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
    id: ID,
    code: Option<AccountCode>,
    name: String,

    #[graphql(skip)]
    entity: Arc<DomainLedgerAccount>,
    #[graphql(skip)]
    from: NaiveDate,
    #[graphql(skip)]
    until: Option<NaiveDate>,
}

impl BalanceSheetAccountSet {
    pub fn new(account: DomainLedgerAccount, from: NaiveDate, until: Option<NaiveDate>) -> Self {
        Self {
            id: account.id.to_global_id(),
            code: account.code.as_ref().map(|code| code.into()),
            name: account.name.clone(),
            entity: Arc::new(account),
            from,
            until,
        }
    }
}

#[ComplexObject]
impl BalanceSheetAccountSet {
    async fn balance_range(&self) -> async_graphql::Result<LedgerAccountBalanceRange> {
        if let Some(balance) = self.entity.btc_balance_range.as_ref() {
            Ok(Some(balance).into())
        } else {
            Ok(self.entity.usd_balance_range.as_ref().into())
        }
    }

    async fn children(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<BalanceSheetAccountSet>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let keys = self
            .entity
            .children_ids
            .iter()
            .copied()
            .map(|id| BalanceSheetAccountSetKey {
                id,
                from: self.from,
                until: self.until,
            })
            .collect::<Vec<_>>();
        let children = loader.load_many(keys.clone()).await?;

        Ok(keys
            .into_iter()
            .filter_map(|key| children.get(&key).cloned())
            .collect())
    }
}
