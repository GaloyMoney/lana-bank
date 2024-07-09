use async_graphql::{dataloader::DataLoader, *};

use crate::server::shared_graphql::{objects::PaginationKey, primitives::UUID};

use super::{account::AccountBalancesByCurrency, loader::ChartOfAccountsLoader};

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

#[derive(SimpleObject, Debug, Clone)]
pub struct AccountSetDetails {
    pub name: String,
    pub id: UUID,
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsAccountSet> for AccountSetDetails {
    fn from(account_set: crate::ledger::account_set::LedgerChartOfAccountsAccountSet) -> Self {
        AccountSetDetails {
            id: account_set.id.into(),
            name: account_set.name,
        }
    }
}

#[derive(Union, Debug, Clone)]
enum ChartOfAccountsCategorySubAccount {
    Account(super::account::AccountDetails),
    AccountSet(AccountSetDetails),
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsCategorySubAccount>
    for ChartOfAccountsCategorySubAccount
{
    fn from(member: crate::ledger::account_set::LedgerChartOfAccountsCategorySubAccount) -> Self {
        match member {
            crate::ledger::account_set::LedgerChartOfAccountsCategorySubAccount::Account(val) => {
                ChartOfAccountsCategorySubAccount::Account(super::account::AccountDetails::from(
                    val,
                ))
            }
            crate::ledger::account_set::LedgerChartOfAccountsCategorySubAccount::AccountSet(
                val,
            ) => ChartOfAccountsCategorySubAccount::AccountSet(AccountSetDetails::from(val)),
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ChartOfAccountsCategoryAccountSet {
    id: UUID,
    name: String,
    has_sub_accounts: bool,
    sub_accounts: Vec<ChartOfAccountsCategorySubAccount>,
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
            sub_accounts: account_set
                .sub_accounts
                .members
                .iter()
                .map(|member| ChartOfAccountsCategorySubAccount::from(member.clone()))
                .collect(),
        }
    }
}

#[derive(Union, Debug, Clone)]
pub enum ChartOfAccountsCategoryAccount {
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
#[graphql(complex)]
pub struct ChartOfAccountsCategory {
    id: UUID,
    name: String,
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsCategory> for ChartOfAccountsCategory {
    fn from(account_set: crate::ledger::account_set::LedgerChartOfAccountsCategory) -> Self {
        ChartOfAccountsCategory {
            id: account_set.id.into(),
            name: account_set.name,
        }
    }
}

#[ComplexObject]
impl ChartOfAccountsCategory {
    async fn accounts(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Vec<ChartOfAccountsCategoryAccount>> {
        let loader = ctx.data_unchecked::<DataLoader<ChartOfAccountsLoader>>();
        let key = PaginationKey {
            key: self.id.clone(),
            first,
            after,
        };
        if let Some(accounts) = loader.load_one(key).await? {
            return Ok(accounts);
        }
        Ok(Vec::new())
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
