use async_graphql::*;

use crate::{app::LavaApp, server::shared_graphql::primitives::UUID};

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
pub struct AccountSetDetails {
    pub id: UUID,
    pub name: String,
    pub has_sub_accounts: bool,
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsAccountSet> for AccountSetDetails {
    fn from(account_set: crate::ledger::account_set::LedgerChartOfAccountsAccountSet) -> Self {
        AccountSetDetails {
            id: account_set.id.into(),
            name: account_set.name,
            has_sub_accounts: account_set.has_sub_accounts,
        }
    }
}

#[derive(Union)]
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

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ChartOfAccountsCategoryAccountWithSubAccounts {
    id: UUID,
    name: String,
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsCategoryAccountSet>
    for ChartOfAccountsCategoryAccountWithSubAccounts
{
    fn from(
        account_set: crate::ledger::account_set::LedgerChartOfAccountsCategoryAccountSet,
    ) -> Self {
        ChartOfAccountsCategoryAccountWithSubAccounts {
            id: account_set.id.into(),
            name: account_set.name,
        }
    }
}

#[ComplexObject]
impl ChartOfAccountsCategoryAccountWithSubAccounts {
    async fn sub_accounts(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Vec<ChartOfAccountsCategorySubAccount>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let account_set = app
            .ledger()
            .chart_of_accounts_category_account_set(self.id.clone().into(), first.into(), after)
            .await?;

        let sub_accounts = if let Some(account_set) = account_set {
            account_set
                .sub_accounts
                .members
                .into_iter()
                .map(ChartOfAccountsCategorySubAccount::from)
                .collect()
        } else {
            Vec::new()
        };

        Ok(sub_accounts)
    }
}

#[derive(Union)]
enum ChartOfAccountsCategoryAccount {
    Account(super::account::AccountDetails),
    AccountSet(AccountSetDetails),
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsCategoryAccount>
    for ChartOfAccountsCategoryAccount
{
    fn from(
        category_account: crate::ledger::account_set::LedgerChartOfAccountsCategoryAccount,
    ) -> Self {
        match category_account {
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
                .into_iter()
                .map(ChartOfAccountsCategoryAccount::from)
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
                .into_iter()
                .map(AccountSetMemberBalance::from)
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
