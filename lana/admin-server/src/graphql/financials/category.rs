use async_graphql::*;

use super::account_set::*;
use crate::graphql::account::*;

#[derive(SimpleObject)]
pub struct ChartOfAccounts {
    name: String,
    categories: Vec<StatementCategory>,
}

// impl From<lana_app::ledger::account_set::LedgerChartOfAccounts> for ChartOfAccounts {
//     fn from(chart_of_accounts: lana_app::ledger::account_set::LedgerChartOfAccounts) -> Self {
//         ChartOfAccounts {
//             name: chart_of_accounts.name,
//             categories: chart_of_accounts
//                 .categories
//                 .into_iter()
//                 .map(StatementCategory::from)
//                 .collect(),
//         }
//     }
// }

#[derive(SimpleObject)]
pub struct StatementCategory {
    name: String,
    amounts: AccountAmountsByCurrency,
    accounts: Vec<AccountSetSubAccount>,
}

impl From<lana_app::statement::StatementAccountSetWithAccounts> for StatementCategory {
    fn from(account_set: lana_app::statement::StatementAccountSetWithAccounts) -> Self {
        StatementCategory {
            name: account_set.name.to_string(),
            amounts: account_set.clone().into(),
            accounts: account_set
                .accounts
                .into_iter()
                .map(AccountSetSubAccount::from)
                .collect(),
        }
    }
}
