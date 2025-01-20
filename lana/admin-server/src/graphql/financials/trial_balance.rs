use async_graphql::*;

use super::account_set::*;
use crate::graphql::account::AccountAmountsByCurrency;

#[derive(SimpleObject)]
pub struct TrialBalance {
    name: String,
    total: AccountAmountsByCurrency,
    sub_accounts: Vec<AccountSetSubAccount>,
}

// impl From<lana_app::ledger::account_set::LedgerTrialBalance> for TrialBalance {
//     fn from(trial_balance: lana_app::ledger::account_set::LedgerTrialBalance) -> Self {
//         TrialBalance {
//             name: trial_balance.name,
//             total: trial_balance.balance.into(),
//             sub_accounts: trial_balance
//                 .accounts
//                 .into_iter()
//                 .map(AccountSetSubAccount::from)
//                 .collect(),
//         }
//     }
// }
