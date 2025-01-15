use async_graphql::*;

use super::category::*;
use crate::graphql::account::*;

#[derive(SimpleObject)]
pub struct BalanceSheet {
    name: String,
    balance: AccountAmountsByCurrency,
    categories: Vec<StatementCategory>,
}

impl From<lana_app::ledger::account_set::LedgerBalanceSheet> for BalanceSheet {
    fn from(balance_sheet: lana_app::ledger::account_set::LedgerBalanceSheet) -> Self {
        BalanceSheet {
            name: balance_sheet.name,
            balance: balance_sheet.balance.into(),
            categories: balance_sheet
                .categories
                .into_iter()
                .map(StatementCategory::from)
                .collect(),
        }
    }
}
