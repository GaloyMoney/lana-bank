use async_graphql::*;

use super::category::*;
use crate::graphql::account::*;

#[derive(SimpleObject)]
pub struct ProfitAndLossStatement {
    name: String,
    net: AccountAmountsByCurrency,
    categories: Vec<StatementCategory>,
}

impl From<lana_app::profit_and_loss::ProfitAndLossStatement> for ProfitAndLossStatement {
    fn from(profit_and_loss: lana_app::profit_and_loss::ProfitAndLossStatement) -> Self {
        ProfitAndLossStatement {
            name: profit_and_loss.name.to_string(),
            net: profit_and_loss.clone().into(),
            categories: profit_and_loss
                .categories
                .into_iter()
                .map(StatementCategory::from)
                .collect(),
        }
    }
}
