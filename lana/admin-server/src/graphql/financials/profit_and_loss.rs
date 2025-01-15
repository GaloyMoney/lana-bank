use async_graphql::*;

use super::category::*;
use crate::graphql::account::*;

#[derive(SimpleObject)]
pub struct ProfitAndLossStatement {
    name: String,
    net: AccountAmountsByCurrency,
    categories: Vec<StatementCategory>,
}

impl From<lana_app::ledger::account_set::LedgerProfitAndLossStatement> for ProfitAndLossStatement {
    fn from(profit_and_loss: lana_app::ledger::account_set::LedgerProfitAndLossStatement) -> Self {
        ProfitAndLossStatement {
            name: profit_and_loss.name,
            net: profit_and_loss.balance.into(),
            categories: profit_and_loss
                .categories
                .into_iter()
                .map(StatementCategory::from)
                .collect(),
        }
    }
}
