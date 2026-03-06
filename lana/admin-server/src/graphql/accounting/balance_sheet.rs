use async_graphql::*;
use chrono::NaiveDate;
use std::collections::HashSet;

use lana_app::balance_sheet::{
    BalanceSheet as DomainBalanceSheet, BalanceSheetAccountSet as DomainBalanceSheetAccountSet,
};

use super::{AccountCode, LedgerAccountBalanceByCurrency};
use crate::{
    graphql::loader::{BalanceSheetAccountKey, LanaDataLoader},
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

    async fn rows(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<BalanceSheetRow>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let mut rows = Vec::new();
        let mut visited = HashSet::new();
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
            })
            .filter(|node| visited.insert(node.key.id))
            .collect::<Vec<_>>();

        while !frontier.is_empty() {
            let keys = frontier.iter().map(|node| node.key).collect::<Vec<_>>();
            let accounts = loader.load_many(keys).await?;
            let mut next_frontier = Vec::new();

            for node in frontier {
                let Some(account) = accounts.get(&node.key).cloned() else {
                    continue;
                };
                let account_id = account.id;
                let child_ids = account.children_ids.clone();
                rows.push(BalanceSheetRow::from_account(account, node.parent_id));

                next_frontier.extend(child_ids.into_iter().filter(|id| visited.insert(*id)).map(
                    |id| PendingBalanceSheetNode {
                        key: BalanceSheetAccountKey {
                            id,
                            as_of: self.as_of,
                        },
                        parent_id: Some(account_id),
                    },
                ));
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
    name: String,
    balance: LedgerAccountBalanceByCurrency,
}

impl BalanceSheetRow {
    fn from_account(
        account: DomainBalanceSheetAccountSet,
        parent_id: Option<lana_app::accounting::LedgerAccountId>,
    ) -> Self {
        Self {
            balance_sheet_account_id: account.id.to_global_id(),
            parent_balance_sheet_account_id: parent_id.map(|id| id.to_global_id()),
            ledger_account_id: UUID::from(account.id),
            code: account.code.as_ref().map(|code| code.into()),
            name: account.name,
            balance: (&account.balance).into(),
        }
    }
}

#[derive(Clone)]
struct PendingBalanceSheetNode {
    key: BalanceSheetAccountKey,
    parent_id: Option<lana_app::accounting::LedgerAccountId>,
}
