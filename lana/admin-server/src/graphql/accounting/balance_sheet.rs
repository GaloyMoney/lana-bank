use async_graphql::*;
use chrono::NaiveDate;

use lana_app::balance_sheet::{
    BalanceSheet as DomainBalanceSheet, BalanceSheetAccountSet as DomainBalanceSheetAccountSet,
};

use super::{AccountCode, LedgerAccountBalanceByCurrency};
use crate::{
    graphql::loader::{BalanceSheetAccountKey, LanaDataLoader},
    primitives::*,
};

const MAX_STATEMENT_TREE_DEPTH: usize = 4;

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
            .map(|id| BalanceSheetAccountKey {
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

    async fn rows(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<BalanceSheetRow>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();

        let mut rows = Vec::new();
        let mut frontier = self
            .entity
            .category_ids
            .iter()
            .copied()
            .map(|id| PendingBalanceSheetNode {
                key: BalanceSheetAccountKey {
                    id,
                    as_of: self.as_of,
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
                rows.push(BalanceSheetRow::new(
                    &account,
                    node.parent_id,
                    category.clone(),
                    node.depth,
                ));

                if node.depth >= MAX_STATEMENT_TREE_DEPTH {
                    continue;
                }

                for child_id in &account.entity.children_ids {
                    next_frontier.push(PendingBalanceSheetNode {
                        key: BalanceSheetAccountKey {
                            id: *child_id,
                            as_of: self.as_of,
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
pub struct BalanceSheetRow {
    balance_sheet_account_id: ID,
    parent_balance_sheet_account_id: Option<ID>,
    ledger_account_id: UUID,
    code: Option<AccountCode>,
    category: String,
    name: String,
    depth: i32,
    balance: LedgerAccountBalanceByCurrency,
}

impl BalanceSheetRow {
    fn new(
        account: &BalanceSheetAccount,
        parent_id: Option<lana_app::accounting::LedgerAccountId>,
        category: String,
        depth: usize,
    ) -> Self {
        Self {
            balance_sheet_account_id: account.entity.id.to_global_id(),
            parent_balance_sheet_account_id: parent_id.map(|id| id.to_global_id()),
            ledger_account_id: UUID::from(account.entity.id),
            code: account.code.clone(),
            category,
            name: account.name.clone(),
            depth: depth as i32,
            balance: (&account.entity.balance).into(),
        }
    }
}

#[derive(Clone)]
struct PendingBalanceSheetNode {
    key: BalanceSheetAccountKey,
    parent_id: Option<lana_app::accounting::LedgerAccountId>,
    category: Option<String>,
    depth: usize,
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct BalanceSheetAccount {
    balance_sheet_account_id: ID,
    ledger_account_id: UUID,
    code: Option<AccountCode>,
    name: String,

    #[graphql(skip)]
    entity: Arc<DomainBalanceSheetAccountSet>,
    #[graphql(skip)]
    as_of: NaiveDate,
    #[graphql(skip)]
    depth: usize,
}

impl BalanceSheetAccount {
    pub fn new(account: DomainBalanceSheetAccountSet, as_of: NaiveDate) -> Self {
        Self::with_depth(account, as_of, 1)
    }

    fn with_depth(account: DomainBalanceSheetAccountSet, as_of: NaiveDate, depth: usize) -> Self {
        Self {
            balance_sheet_account_id: account.id.to_global_id(),
            ledger_account_id: UUID::from(account.id),
            code: account.code.as_ref().map(|code| code.into()),
            name: account.name.clone(),
            entity: Arc::new(account),
            as_of,
            depth,
        }
    }
}

#[ComplexObject]
impl BalanceSheetAccount {
    async fn balance(&self) -> async_graphql::Result<LedgerAccountBalanceByCurrency> {
        Ok((&self.entity.balance).into())
    }

    async fn children(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<BalanceSheetAccount>> {
        if self.depth >= MAX_STATEMENT_TREE_DEPTH {
            return Ok(Vec::new());
        }

        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let keys = self
            .entity
            .children_ids
            .iter()
            .copied()
            .map(|id| BalanceSheetAccountKey {
                id,
                as_of: self.as_of,
            })
            .collect::<Vec<_>>();
        let children = loader.load_many(keys.clone()).await?;

        Ok(keys
            .into_iter()
            .filter_map(|key| children.get(&key).cloned())
            .map(|child| {
                BalanceSheetAccount::with_depth(
                    child.entity.as_ref().clone(),
                    child.as_of,
                    self.depth + 1,
                )
            })
            .collect())
    }
}
