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

// impl From<lana_app::ledger::account_set::LedgerStatementCategoryWithBalance> for StatementCategory {
//     fn from(
//         account_set: lana_app::ledger::account_set::LedgerStatementCategoryWithBalance,
//     ) -> Self {
//         StatementCategory {
//             name: account_set.name,
//             amounts: account_set.balance.into(),
//             accounts: account_set
//                 .accounts
//                 .into_iter()
//                 .map(AccountSetSubAccount::from)
//                 .collect(),
//         }
//     }
// }
