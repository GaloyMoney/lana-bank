use async_graphql::*;

use super::category::*;
use crate::graphql::account::*;

#[derive(SimpleObject)]
pub struct BalanceSheet {
    name: String,
    balance: AccountAmountsByCurrency,
    categories: Vec<StatementCategory>,
}

impl From<lana_app::balance_sheet::BalanceSheet> for BalanceSheet {
    fn from(balance_sheet: lana_app::balance_sheet::BalanceSheet) -> Self {
        BalanceSheet {
            name: balance_sheet.name.to_string(),
            balance: balance_sheet.clone().into(),
            categories: balance_sheet
                .categories
                .into_iter()
                .map(StatementCategory::from)
                .collect(),
        }
    }
}
