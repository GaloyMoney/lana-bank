use async_graphql::*;

use super::category::*;
use crate::graphql::account::*;

#[derive(SimpleObject)]
pub struct CashFlowStatement {
    name: String,
    total: AccountAmountsByCurrency,
    categories: Vec<StatementCategory>,
}

impl From<lana_app::cash_flow::CashFlowStatement> for CashFlowStatement {
    fn from(cash_flow: lana_app::cash_flow::CashFlowStatement) -> Self {
        CashFlowStatement {
            name: cash_flow.name.to_string(),
            total: cash_flow.clone().into(),
            categories: cash_flow
                .categories
                .into_iter()
                .map(StatementCategory::from)
                .collect(),
        }
    }
}
