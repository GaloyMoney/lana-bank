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

const MAX_STATEMENT_TREE_DEPTH: usize = 4;

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

    async fn rows(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<ProfitAndLossRow>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();

        let mut rows = Vec::new();
        let mut frontier = self
            .entity
            .category_ids
            .iter()
            .copied()
            .map(|id| PendingProfitAndLossNode {
                key: ProfitAndLossAccountKey {
                    id,
                    from: self.from,
                    until: self.until,
                },
                parent_id: None,
                category: None,
                depth: 1,
            })
            .collect::<Vec<_>>();

        while !frontier.is_empty() {
            let keys = frontier.iter().map(|node| node.key).collect::<Vec<_>>();
            let accounts = loader.load_many(keys).await?;
            let mut next_frontier = Vec::new();

            for node in frontier {
                let Some(account) = accounts.get(&node.key).cloned() else {
                    continue;
                };

                let category = node.category.unwrap_or_else(|| account.name.clone());
                rows.push(ProfitAndLossRow::new(
                    &account,
                    node.parent_id,
                    category.clone(),
                    node.depth,
                ));

                if node.depth >= MAX_STATEMENT_TREE_DEPTH {
                    continue;
                }

                for child_id in &account.entity.children_ids {
                    next_frontier.push(PendingProfitAndLossNode {
                        key: ProfitAndLossAccountKey {
                            id: *child_id,
                            from: self.from,
                            until: self.until,
                        },
                        parent_id: Some(account.entity.id),
                        category: Some(category.clone()),
                        depth: node.depth + 1,
                    });
                }
            }

            frontier = next_frontier;
        }

        Ok(rows)
    }
}

#[derive(SimpleObject)]
pub struct ProfitAndLossRow {
    profit_and_loss_account_id: ID,
    parent_profit_and_loss_account_id: Option<ID>,
    ledger_account_id: UUID,
    code: Option<AccountCode>,
    category: String,
    name: String,
    depth: i32,
    balance_range: LedgerAccountBalanceRange,
}

impl ProfitAndLossRow {
    fn new(
        account: &ProfitAndLossAccount,
        parent_id: Option<lana_app::accounting::LedgerAccountId>,
        category: String,
        depth: usize,
    ) -> Self {
        let balance_range = if let Some(balance) = account.entity.btc_balance_range.as_ref() {
            Some(balance).into()
        } else {
            account.entity.usd_balance_range.as_ref().into()
        };

        Self {
            profit_and_loss_account_id: account.entity.id.to_global_id(),
            parent_profit_and_loss_account_id: parent_id.map(|id| id.to_global_id()),
            ledger_account_id: UUID::from(account.entity.id),
            code: account.code.clone(),
            category,
            name: account.name.clone(),
            depth: depth as i32,
            balance_range,
        }
    }
}

#[derive(Clone)]
struct PendingProfitAndLossNode {
    key: ProfitAndLossAccountKey,
    parent_id: Option<lana_app::accounting::LedgerAccountId>,
    category: Option<String>,
    depth: usize,
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
    #[graphql(skip)]
    depth: usize,
}

impl ProfitAndLossAccount {
    pub fn new(account: DomainLedgerAccount, from: NaiveDate, until: Option<NaiveDate>) -> Self {
        Self::with_depth(account, from, until, 1)
    }

    fn with_depth(
        account: DomainLedgerAccount,
        from: NaiveDate,
        until: Option<NaiveDate>,
        depth: usize,
    ) -> Self {
        Self {
            profit_and_loss_account_id: account.id.to_global_id(),
            ledger_account_id: UUID::from(account.id),
            code: account.code.as_ref().map(|code| code.into()),
            name: account.name.clone(),
            entity: Arc::new(account),
            from,
            until,
            depth,
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
        if self.depth >= MAX_STATEMENT_TREE_DEPTH {
            return Ok(Vec::new());
        }

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
            .map(|child| {
                ProfitAndLossAccount::with_depth(
                    child.entity.as_ref().clone(),
                    child.from,
                    child.until,
                    self.depth + 1,
                )
            })
            .collect())
    }
}
