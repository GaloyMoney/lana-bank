use async_graphql::{types::connection::*, *};
use serde::{Deserialize, Serialize};

use crate::{app::LavaApp, server::shared_graphql::primitives::UUID};

use super::account::AccountBalancesByCurrency;

#[derive(SimpleObject)]
pub struct AccountSetWithBalance {
    id: UUID,
    name: String,
    balance: AccountBalancesByCurrency,
    has_sub_accounts: bool,
}

impl From<crate::ledger::account_set::LedgerAccountSetWithBalance> for AccountSetWithBalance {
    fn from(line_item: crate::ledger::account_set::LedgerAccountSetWithBalance) -> Self {
        AccountSetWithBalance {
            id: line_item.id.into(),
            name: line_item.name,
            balance: line_item.balance.into(),
            has_sub_accounts: line_item.has_sub_accounts,
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountSetDetails {
    pub id: UUID,
    pub name: String,
    pub has_sub_accounts: bool,
}

impl From<crate::ledger::account_set::LedgerAccountSetDetails> for AccountSetDetails {
    fn from(account_set: crate::ledger::account_set::LedgerAccountSetDetails) -> Self {
        AccountSetDetails {
            id: account_set.id.into(),
            name: account_set.name,
            has_sub_accounts: account_set.page_info.start_cursor.is_some(),
        }
    }
}

#[derive(Union)]
enum AccountSetSubAccount {
    Account(super::account::AccountDetails),
    AccountSet(AccountSetDetails),
}

impl From<crate::ledger::account_set::PaginatedLedgerAccountSetSubAccount>
    for AccountSetSubAccount
{
    fn from(member: crate::ledger::account_set::PaginatedLedgerAccountSetSubAccount) -> Self {
        match member.value {
            crate::ledger::account_set::LedgerAccountSetSubAccount::Account(val) => {
                AccountSetSubAccount::Account(super::account::AccountDetails::from(val))
            }
            crate::ledger::account_set::LedgerAccountSetSubAccount::AccountSet(val) => {
                AccountSetSubAccount::AccountSet(AccountSetDetails::from(val))
            }
        }
    }
}

impl From<crate::ledger::account_set::LedgerAccountSetSubAccount> for AccountSetSubAccount {
    fn from(category_account: crate::ledger::account_set::LedgerAccountSetSubAccount) -> Self {
        match category_account {
            crate::ledger::account_set::LedgerAccountSetSubAccount::Account(val) => {
                AccountSetSubAccount::Account(val.into())
            }
            crate::ledger::account_set::LedgerAccountSetSubAccount::AccountSet(val) => {
                AccountSetSubAccount::AccountSet(val.into())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct SubAccountCursor {
    pub value: String,
}

impl CursorType for SubAccountCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        self.value.clone()
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        Ok(SubAccountCursor {
            value: s.to_string(),
        })
    }
}

impl From<String> for SubAccountCursor {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl From<SubAccountCursor> for crate::ledger::cursor::SubAccountCursor {
    fn from(cursor: SubAccountCursor) -> Self {
        Self {
            value: cursor.value,
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct AccountSetAndSubAccounts {
    id: UUID,
    name: String,
}

impl From<crate::ledger::account_set::LedgerAccountSetAndSubAccounts> for AccountSetAndSubAccounts {
    fn from(account_set: crate::ledger::account_set::LedgerAccountSetAndSubAccounts) -> Self {
        AccountSetAndSubAccounts {
            id: account_set.id.into(),
            name: account_set.name,
        }
    }
}

#[ComplexObject]
impl AccountSetAndSubAccounts {
    async fn sub_accounts(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> Result<Connection<SubAccountCursor, AccountSetSubAccount, EmptyFields, EmptyFields>> {
        let app = ctx.data_unchecked::<LavaApp>();
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .ledger()
                    .paginated_account_set_and_sub_accounts(
                        self.id.clone().into(),
                        crate::query::PaginatedQueryArgs {
                            first,
                            after: after.map(crate::ledger::SubAccountCursor::from),
                        },
                    )
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|sub_account| {
                        let cursor = SubAccountCursor::from(sub_account.cursor.clone());
                        Edge::new(cursor, AccountSetSubAccount::from(sub_account))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}

#[derive(Union)]
enum AccountSetSubAccountWithBalance {
    Account(super::account::AccountWithBalance),
    AccountSet(AccountSetWithBalance),
}

impl From<crate::ledger::account_set::PaginatedLedgerAccountSetSubAccountWithBalance>
    for AccountSetSubAccountWithBalance
{
    fn from(
        member: crate::ledger::account_set::PaginatedLedgerAccountSetSubAccountWithBalance,
    ) -> Self {
        match member.value {
            crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance::Account(val) => {
                AccountSetSubAccountWithBalance::Account(super::account::AccountWithBalance::from(
                    val,
                ))
            }
            crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance::AccountSet(val) => {
                AccountSetSubAccountWithBalance::AccountSet(AccountSetWithBalance::from(val))
            }
        }
    }
}

impl From<crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance>
    for AccountSetSubAccountWithBalance
{
    fn from(member: crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance) -> Self {
        match member {
            crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance::Account(val) => {
                AccountSetSubAccountWithBalance::Account(super::account::AccountWithBalance::from(
                    val,
                ))
            }
            crate::ledger::account_set::LedgerAccountSetSubAccountWithBalance::AccountSet(val) => {
                AccountSetSubAccountWithBalance::AccountSet(AccountSetWithBalance::from(val))
            }
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct AccountSetAndSubAccountsWithBalance {
    id: UUID,
    name: String,
    balance: AccountBalancesByCurrency,
}

impl From<crate::ledger::account_set::LedgerAccountSetAndSubAccountsWithBalance>
    for AccountSetAndSubAccountsWithBalance
{
    fn from(
        account_set: crate::ledger::account_set::LedgerAccountSetAndSubAccountsWithBalance,
    ) -> Self {
        AccountSetAndSubAccountsWithBalance {
            id: account_set.id.into(),
            name: account_set.name,
            balance: account_set.balance.into(),
        }
    }
}

#[ComplexObject]
impl AccountSetAndSubAccountsWithBalance {
    async fn sub_accounts(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> Result<
        Connection<SubAccountCursor, AccountSetSubAccountWithBalance, EmptyFields, EmptyFields>,
    > {
        let app = ctx.data_unchecked::<LavaApp>();
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .ledger()
                    .paginated_account_set_and_sub_accounts_with_balance(
                        self.id.clone().into(),
                        crate::query::PaginatedQueryArgs {
                            first,
                            after: after.map(crate::ledger::SubAccountCursor::from),
                        },
                    )
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|sub_account| {
                        let cursor = SubAccountCursor::from(sub_account.cursor.clone());
                        Edge::new(cursor, AccountSetSubAccountWithBalance::from(sub_account))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}
#[derive(SimpleObject)]
pub struct ChartOfAccountsCategory {
    name: String,
    accounts: Vec<AccountSetSubAccount>,
}

impl From<crate::ledger::account_set::LedgerChartOfAccountsCategory> for ChartOfAccountsCategory {
    fn from(account_set: crate::ledger::account_set::LedgerChartOfAccountsCategory) -> Self {
        ChartOfAccountsCategory {
            name: account_set.name,
            accounts: account_set
                .category_accounts
                .into_iter()
                .map(AccountSetSubAccount::from)
                .collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct TrialBalance {
    name: String,
    balance: AccountBalancesByCurrency,
    sub_accounts: Vec<AccountSetSubAccountWithBalance>,
}

impl From<crate::ledger::account_set::LedgerTrialBalance> for TrialBalance {
    fn from(trial_balance: crate::ledger::account_set::LedgerTrialBalance) -> Self {
        TrialBalance {
            name: trial_balance.name,
            balance: trial_balance.balance.into(),
            sub_accounts: trial_balance
                .accounts
                .into_iter()
                .map(AccountSetSubAccountWithBalance::from)
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
