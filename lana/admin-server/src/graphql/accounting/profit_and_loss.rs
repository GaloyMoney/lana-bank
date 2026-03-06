use async_graphql::*;
use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};

use lana_app::accounting::ledger_account::LedgerAccount as DomainLedgerAccount;
use lana_app::profit_and_loss::ProfitAndLossStatement as DomainProfitAndLossStatement;

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

    async fn rows(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<ProfitAndLossRow>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let mut rows = Vec::new();
        let mut visited = HashSet::new();
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
            })
            .filter(|node| visited.insert(node.key.id))
            .collect::<Vec<_>>();

        while !frontier.is_empty() {
            let keys = frontier.iter().map(|node| node.key).collect::<Vec<_>>();
            let accounts: HashMap<ProfitAndLossAccountKey, DomainLedgerAccount> =
                loader.load_many(keys).await?;
            let mut next_frontier = Vec::new();

            for node in frontier {
                let Some(account) = accounts.get(&node.key).cloned() else {
                    continue;
                };
                let account_id = account.id;
                let child_ids = account.children_ids.clone();
                rows.push(ProfitAndLossRow::from_account(account, node.parent_id));

                next_frontier.extend(child_ids.into_iter().filter(|id| visited.insert(*id)).map(
                    |id| PendingProfitAndLossNode {
                        key: ProfitAndLossAccountKey {
                            id,
                            from: self.from,
                            until: self.until,
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
pub struct ProfitAndLossRow {
    profit_and_loss_account_id: ID,
    parent_profit_and_loss_account_id: Option<ID>,
    ledger_account_id: UUID,
    code: Option<AccountCode>,
    name: String,
    balance_range: LedgerAccountBalanceRange,
}

impl ProfitAndLossRow {
    fn from_account(
        account: DomainLedgerAccount,
        parent_id: Option<lana_app::accounting::LedgerAccountId>,
    ) -> Self {
        let balance_range = if let Some(balance) = account.btc_balance_range.as_ref() {
            Some(balance).into()
        } else {
            account.usd_balance_range.as_ref().into()
        };

        Self {
            profit_and_loss_account_id: account.id.to_global_id(),
            parent_profit_and_loss_account_id: parent_id.map(|id| id.to_global_id()),
            ledger_account_id: UUID::from(account.id),
            code: account.code.as_ref().map(AccountCode::from),
            name: account.name,
            balance_range,
        }
    }
}

#[derive(Clone)]
struct PendingProfitAndLossNode {
    key: ProfitAndLossAccountKey,
    parent_id: Option<lana_app::accounting::LedgerAccountId>,
}
