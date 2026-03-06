use async_graphql::*;
use chrono::NaiveDate;

use lana_app::{
    accounting::ledger_account::LedgerAccount as DomainLedgerAccount,
    profit_and_loss::ProfitAndLossStatement as DomainProfitAndLossStatement,
};

use crate::{
    graphql::loader::{LanaDataLoader, ProfitAndLossAccountKey},
    primitives::*,
};

use super::{
    AccountCode, BtcLedgerAccountBalanceRange, LedgerAccountBalanceRange,
    LedgerAccountBalanceRangeByCurrency, UsdLedgerAccountBalanceRange,
};

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct ProfitAndLossStatement {
    pub name: String,
    #[graphql(skip)]
    pub entity: Arc<DomainProfitAndLossStatement>,
    #[graphql(skip)]
    from: NaiveDate,
    #[graphql(skip)]
    until: Option<NaiveDate>,
}

impl ProfitAndLossStatement {
    pub fn new(
        profit_and_loss: DomainProfitAndLossStatement,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Self {
        Self {
            name: profit_and_loss.name.to_string(),
            entity: Arc::new(profit_and_loss),
            from,
            until,
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

    async fn categories(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<ProfitAndLossAccount>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let keys = self
            .entity
            .category_ids
            .iter()
            .copied()
            .map(|id| ProfitAndLossAccountKey {
                id,
                from: self.from,
                until: self.until,
            })
            .collect::<Vec<_>>();
        let categories = loader.load_many(keys.clone()).await?;

        Ok(keys
            .into_iter()
            .filter_map(|id| categories.get(&id).cloned())
            .collect())
    }
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct ProfitAndLossAccount {
    profit_and_loss_account_id: ID,
    ledger_account_id: UUID,
    code: Option<AccountCode>,
    name: String,

    #[graphql(skip)]
    entity: Arc<DomainLedgerAccount>,
    #[graphql(skip)]
    from: NaiveDate,
    #[graphql(skip)]
    until: Option<NaiveDate>,
}

impl ProfitAndLossAccount {
    pub fn new(account: DomainLedgerAccount, from: NaiveDate, until: Option<NaiveDate>) -> Self {
        Self {
            profit_and_loss_account_id: account.id.to_global_id(),
            ledger_account_id: UUID::from(account.id),
            code: account.code.as_ref().map(|code| code.into()),
            name: account.name.clone(),
            entity: Arc::new(account),
            from,
            until,
        }
    }
}

#[ComplexObject]
impl ProfitAndLossAccount {
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
    ) -> async_graphql::Result<Vec<ProfitAndLossAccount>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let keys = self
            .entity
            .children_ids
            .iter()
            .copied()
            .map(|id| ProfitAndLossAccountKey {
                id,
                from: self.from,
                until: self.until,
            })
            .collect::<Vec<_>>();
        let children = loader.load_many(keys.clone()).await?;

        Ok(keys
            .into_iter()
            .filter_map(|id| children.get(&id).cloned())
            .collect())
    }
}
