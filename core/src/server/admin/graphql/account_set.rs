use async_graphql::*;

use crate::server::shared_graphql::primitives::UUID;

use super::account::AccountBalancesByCurrency;

#[derive(SimpleObject)]
pub struct AccountSetBalance {
    name: String,
    balance: AccountBalancesByCurrency,
}

impl From<crate::ledger::account_set::LedgerAccountSetBalance> for AccountSetBalance {
    fn from(line_item: crate::ledger::account_set::LedgerAccountSetBalance) -> Self {
        AccountSetBalance {
            name: line_item.name,
            balance: line_item.balance.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct ChartOfAccountsCategoryAccountSet {
    id: UUID,
    name: String,
    has_sub_accounts: bool,
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsCategoryAccountSet>
    for ChartOfAccountsCategoryAccountSet
{
    fn from(
        account_set: crate::ledger::account_set::LedgerChartOfAccountsCategoryAccountSet,
    ) -> Self {
        ChartOfAccountsCategoryAccountSet {
            id: account_set.id.into(),
            name: account_set.name,
            has_sub_accounts: account_set.sub_accounts.has_sub_accounts,
        }
    }
}

#[derive(Union)]
enum ChartOfAccountsCategoryAccount {
    Account(super::account::AccountDetails),
    AccountSet(ChartOfAccountsCategoryAccountSet),
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsCategoryAccount>
    for ChartOfAccountsCategoryAccount
{
    fn from(
        member_balance: crate::ledger::account_set::LedgerChartOfAccountsCategoryAccount,
    ) -> Self {
        match member_balance {
            crate::ledger::account_set::LedgerChartOfAccountsCategoryAccount::Account(val) => {
                ChartOfAccountsCategoryAccount::Account(val.into())
            }
            crate::ledger::account_set::LedgerChartOfAccountsCategoryAccount::AccountSet(val) => {
                ChartOfAccountsCategoryAccount::AccountSet(val.into())
            }
        }
    }
}

#[derive(SimpleObject)]
pub struct ChartOfAccountsCategory {
    id: UUID,
    name: String,
    accounts: Vec<ChartOfAccountsCategoryAccount>,
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsCategory> for ChartOfAccountsCategory {
    fn from(account_set: crate::ledger::account_set::LedgerChartOfAccountsCategory) -> Self {
        ChartOfAccountsCategory {
            id: account_set.id.into(),
            name: account_set.name,
            accounts: account_set
                .category_accounts
                .iter()
                .map(|m| ChartOfAccountsCategoryAccount::from(m.clone()))
                .collect(),
        }
    }
}

#[derive(Union)]
enum AccountSetMemberBalance {
    Account(super::account::AccountBalance),
    AccountSet(AccountSetBalance),
}

impl From<crate::ledger::account_set::LedgerAccountSetMemberBalance> for AccountSetMemberBalance {
    fn from(member_balance: crate::ledger::account_set::LedgerAccountSetMemberBalance) -> Self {
        match member_balance {
            crate::ledger::account_set::LedgerAccountSetMemberBalance::LedgerAccountBalance(
                val,
            ) => AccountSetMemberBalance::Account(val.into()),
            crate::ledger::account_set::LedgerAccountSetMemberBalance::LedgerAccountSetBalance(
                val,
            ) => AccountSetMemberBalance::AccountSet(val.into()),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountSetAndMemberBalances {
    name: String,
    balance: AccountBalancesByCurrency,
    member_balances: Vec<AccountSetMemberBalance>,
}

impl From<crate::ledger::account_set::LedgerAccountSetAndMemberBalances>
    for AccountSetAndMemberBalances
{
    fn from(trial_balance: crate::ledger::account_set::LedgerAccountSetAndMemberBalances) -> Self {
        AccountSetAndMemberBalances {
            name: trial_balance.name,
            balance: trial_balance.balance.into(),
            member_balances: trial_balance
                .member_balances
                .iter()
                .map(|l| AccountSetMemberBalance::from(l.clone()))
                .collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct ChartOfAccounts {
    name: String,
    categories: Vec<ChartOfAccountsCategory>,
}

impl From<crate::ledger::account_set::LedgerChartOfAccounts> for ChartOfAccounts {
    fn from(chart_of_accounts: crate::ledger::account_set::LedgerChartOfAccounts) -> Self {
        ChartOfAccounts {
            name: chart_of_accounts.name,
            categories: chart_of_accounts
                .categories
                .iter()
                .map(|category| category.clone().into())
                .collect(),
        }
    }
}
