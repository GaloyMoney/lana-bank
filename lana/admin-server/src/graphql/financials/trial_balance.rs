use async_graphql::*;

use super::account_set::*;
use crate::graphql::account::AccountAmountsByCurrency;

#[derive(SimpleObject)]
pub struct TrialBalance {
    name: String,
    total: AccountAmountsByCurrency,
    sub_accounts: Vec<AccountSetSubAccount>,
}

impl From<lana_app::trial_balance::TrialBalance> for TrialBalance {
    fn from(trial_balance: lana_app::trial_balance::TrialBalance) -> Self {
        TrialBalance {
            name: trial_balance.name.to_string(),
            total: trial_balance.clone().into(),
            sub_accounts: trial_balance
                .accounts
                .into_iter()
                .map(AccountSetSubAccount::from)
                .collect(),
        }
    }
}
