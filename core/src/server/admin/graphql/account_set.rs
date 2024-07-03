use async_graphql::*;

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

#[derive(Union)]
enum AccountMemberDetails {
    Account(super::account::AccountDetails),
    AccountSet(AccountSetDetails),
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsSubMember> for AccountMemberDetails {
    fn from(member_balance: crate::ledger::account_set::LedgerChartOfAccountsSubMember) -> Self {
        match member_balance {
            crate::ledger::account_set::LedgerChartOfAccountsSubMember::Account(val) => {
                AccountMemberDetails::Account(val.into())
            }
            crate::ledger::account_set::LedgerChartOfAccountsSubMember::AccountSet(val) => {
                AccountMemberDetails::AccountSet(val.into())
            }
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountSetDetails {
    name: String,
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsAccountSet> for AccountSetDetails {
    fn from(account_set: crate::ledger::account_set::LedgerChartOfAccountsAccountSet) -> Self {
        AccountSetDetails {
            name: account_set.name,
        }
    }
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsSubAccountSet> for AccountSetDetails {
    fn from(account_set: crate::ledger::account_set::LedgerChartOfAccountsSubAccountSet) -> Self {
        AccountSetDetails {
            name: account_set.name,
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountSetDetailsAndAccounts {
    name: String,
    accounts: Vec<AccountMemberDetails>,
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsAccountSet>
    for AccountSetDetailsAndAccounts
{
    fn from(account_set: crate::ledger::account_set::LedgerChartOfAccountsAccountSet) -> Self {
        AccountSetDetailsAndAccounts {
            name: account_set.name,
            accounts: account_set
                .members
                .iter()
                .map(|m| AccountMemberDetails::from(m.clone()))
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

#[derive(Union)]
enum ChartOfAccountsCategory {
    Account(super::account::AccountDetails),
    AccountSet(AccountSetDetailsAndAccounts),
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsMember> for ChartOfAccountsCategory {
    fn from(
        chart_of_accounts_type: crate::ledger::account_set::LedgerChartOfAccountsMember,
    ) -> Self {
        match chart_of_accounts_type {
            crate::ledger::account_set::LedgerChartOfAccountsMember::Account(val) => {
                ChartOfAccountsCategory::Account(val.into())
            }
            crate::ledger::account_set::LedgerChartOfAccountsMember::AccountSet(val) => {
                ChartOfAccountsCategory::AccountSet(val.into())
            }
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
                .members
                .iter()
                .map(|account_type| ChartOfAccountsCategory::from(account_type.clone()))
                .collect(),
        }
    }
}
