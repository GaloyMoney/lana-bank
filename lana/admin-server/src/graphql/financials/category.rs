use async_graphql::*;

use super::account_set::*;
use crate::graphql::account::*;

#[derive(SimpleObject)]
pub struct StatementCategory {
    name: String,
    amounts: AccountAmountsByCurrency,
    accounts: Vec<AccountSetSubAccount>,
}

impl From<lana_app::ledger::account_set::LedgerStatementCategoryWithBalance> for StatementCategory {
    fn from(
        account_set: lana_app::ledger::account_set::LedgerStatementCategoryWithBalance,
    ) -> Self {
        StatementCategory {
            name: account_set.name,
            amounts: account_set.balance.into(),
            accounts: account_set
                .accounts
                .into_iter()
                .map(AccountSetSubAccount::from)
                .collect(),
        }
    }
}
