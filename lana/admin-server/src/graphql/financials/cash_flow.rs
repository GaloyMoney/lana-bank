use async_graphql::*;

use super::category::*;
use crate::graphql::account::*;

#[derive(SimpleObject)]
pub struct CashFlowStatement {
    name: String,
    total: AccountAmountsByCurrency,
    categories: Vec<StatementCategory>,
}

impl From<lana_app::ledger::account_set::LedgerCashFlowStatement> for CashFlowStatement {
    fn from(profit_and_loss: lana_app::ledger::account_set::LedgerCashFlowStatement) -> Self {
        CashFlowStatement {
            name: profit_and_loss.name,
            total: profit_and_loss.balance.into(),
            categories: profit_and_loss
                .categories
                .into_iter()
                .map(StatementCategory::from)
                .collect(),
        }
    }
}
