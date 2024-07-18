use crate::primitives::{LedgerAccountSetId, LedgerAccountSetMemberType, LedgerDebitOrCredit};

use super::{account::*, cala::graphql::*};

#[derive(Debug, Clone)]
pub struct LedgerAccountSetWithBalance {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub has_sub_accounts: bool,
}

impl From<trial_balance::TrialBalanceAccountSetMembersEdgesNodeOnAccountSet>
    for LedgerAccountSetWithBalance
{
    fn from(node: trial_balance::TrialBalanceAccountSetMembersEdgesNodeOnAccountSet) -> Self {
        LedgerAccountSetWithBalance {
            id: node.account_set_id.into(),
            name: node.name,
            normal_balance_type: node.normal_balance_type.into(),
            balance: LedgerAccountBalancesByCurrency {
                btc: node.account_set_balances.btc_balances.map_or_else(
                    LayeredBtcAccountBalances::default,
                    LayeredBtcAccountBalances::from,
                ),
                usd: node.account_set_balances.usd_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
                usdt: node.account_set_balances.usdt_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
            },
            has_sub_accounts: node.members.page_info.start_cursor.is_some(),
        }
    }
}

impl From<account_set_and_sub_accounts_with_balance::SubAccountOnAccountSet>
    for LedgerAccountSetWithBalance
{
    fn from(node: account_set_and_sub_accounts_with_balance::SubAccountOnAccountSet) -> Self {
        let account_set = node.account_set_with_balance;

        LedgerAccountSetWithBalance {
            id: account_set.account_set_id.into(),
            name: account_set.name,
            normal_balance_type: account_set.normal_balance_type.into(),
            balance: LedgerAccountBalancesByCurrency {
                btc: account_set.account_set_balances.btc_balances.map_or_else(
                    LayeredBtcAccountBalances::default,
                    LayeredBtcAccountBalances::from,
                ),
                usd: account_set.account_set_balances.usd_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
                usdt: account_set.account_set_balances.usdt_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
            },
            has_sub_accounts: account_set.members.page_info.start_cursor.is_some(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LedgerAccountSetSubAccountWithBalance {
    Account(LedgerAccountWithBalance),
    AccountSet(LedgerAccountSetWithBalance),
}

pub struct LedgerAccountSetAndSubAccountsWithBalance {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub sub_accounts: Vec<LedgerAccountSetSubAccountWithBalance>,
}

impl From<trial_balance::TrialBalanceAccountSet> for LedgerAccountSetAndSubAccountsWithBalance {
    fn from(account_set: trial_balance::TrialBalanceAccountSet) -> Self {
        let sub_accounts: Vec<LedgerAccountSetSubAccountWithBalance> = account_set
            .members
            .edges
            .into_iter()
            .map(|e| match e.node {
                trial_balance::TrialBalanceAccountSetMembersEdgesNode::Account(node) => {
                    LedgerAccountSetSubAccountWithBalance::Account(LedgerAccountWithBalance::from(
                        node,
                    ))
                }
                trial_balance::TrialBalanceAccountSetMembersEdgesNode::AccountSet(node) => {
                    LedgerAccountSetSubAccountWithBalance::AccountSet(
                        LedgerAccountSetWithBalance::from(node),
                    )
                }
            })
            .collect();

        Self {
            name: account_set.name,
            normal_balance_type: account_set.normal_balance_type.into(),
            balance: LedgerAccountBalancesByCurrency {
                btc: account_set.account_set_balances.btc_balances.map_or_else(
                    LayeredBtcAccountBalances::default,
                    LayeredBtcAccountBalances::from,
                ),
                usd: account_set.account_set_balances.usd_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
                usdt: account_set.account_set_balances.usdt_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
            },
            sub_accounts,
        }
    }
}

impl From<account_set_and_sub_accounts_with_balance::AccountSetAndSubAccountsWithBalanceAccountSet>
    for LedgerAccountSetAndSubAccountsWithBalance
{
    fn from(
        account_set: account_set_and_sub_accounts_with_balance::AccountSetAndSubAccountsWithBalanceAccountSet,
    ) -> Self {
        let sub_accounts: Vec<LedgerAccountSetSubAccountWithBalance> = account_set
            .sub_accounts
            .edges
            .iter()
            .map(|e| match &e.node {
                account_set_and_sub_accounts_with_balance::subAccount::Account(node) => {
                    LedgerAccountSetSubAccountWithBalance::Account(LedgerAccountWithBalance::from(
                        node.clone(),
                    ))
                }
                account_set_and_sub_accounts_with_balance::subAccount::AccountSet(node) => {
                    LedgerAccountSetSubAccountWithBalance::AccountSet(
                        LedgerAccountSetWithBalance::from(node.clone()),
                    )
                }
            })
            .collect();

        let account_set_with_balance = account_set.account_set_with_balance;
        LedgerAccountSetAndSubAccountsWithBalance {
            name: account_set_with_balance.name,
            normal_balance_type: account_set_with_balance.normal_balance_type.into(),
            balance: LedgerAccountBalancesByCurrency {
                btc: account_set_with_balance
                    .account_set_balances
                    .btc_balances
                    .map_or_else(
                        LayeredBtcAccountBalances::default,
                        LayeredBtcAccountBalances::from,
                    ),
                usd: account_set_with_balance
                    .account_set_balances
                    .usd_balances
                    .map_or_else(
                        LayeredUsdAccountBalances::default,
                        LayeredUsdAccountBalances::from,
                    ),
                usdt: account_set_with_balance
                    .account_set_balances
                    .usdt_balances
                    .map_or_else(
                        LayeredUsdAccountBalances::default,
                        LayeredUsdAccountBalances::from,
                    ),
            },
            sub_accounts,
        }
    }
}

impl From<account_set_by_id::AccountSetByIdAccountSet> for LedgerAccountSetId {
    fn from(account_set: account_set_by_id::AccountSetByIdAccountSet) -> Self {
        Self::from(account_set.account_set_id)
    }
}

impl From<LedgerAccountSetMemberType> for add_to_account_set::AccountSetMemberType {
    fn from(member_type: LedgerAccountSetMemberType) -> Self {
        match member_type {
            LedgerAccountSetMemberType::Account => Self::ACCOUNT,
            LedgerAccountSetMemberType::AccountSet => Self::ACCOUNT_SET,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PageExistsPageInfo {
    pub start_cursor: Option<String>,
}

impl From<chart_of_accounts::AccountSetDetailsMembersPageInfo> for PageExistsPageInfo {
    fn from(page_info: chart_of_accounts::AccountSetDetailsMembersPageInfo) -> Self {
        PageExistsPageInfo {
            start_cursor: page_info.start_cursor,
        }
    }
}

impl From<account_set_and_sub_accounts::AccountSetDetailsMembersPageInfo> for PageExistsPageInfo {
    fn from(page_info: account_set_and_sub_accounts::AccountSetDetailsMembersPageInfo) -> Self {
        PageExistsPageInfo {
            start_cursor: page_info.start_cursor,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionCreationPageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LedgerAccountSetDetails {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub page_info: PageExistsPageInfo,
}

impl From<chart_of_accounts::accountSetDetails> for LedgerAccountSetDetails {
    fn from(account_set_details: chart_of_accounts::accountSetDetails) -> Self {
        LedgerAccountSetDetails {
            id: account_set_details.account_set_id.into(),
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            page_info: account_set_details.members.page_info.into(),
        }
    }
}

impl From<account_set_and_sub_accounts::accountSetDetails> for LedgerAccountSetDetails {
    fn from(account_set_details: account_set_and_sub_accounts::accountSetDetails) -> Self {
        LedgerAccountSetDetails {
            id: account_set_details.account_set_id.into(),
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            page_info: account_set_details.members.page_info.into(),
        }
    }
}

impl From<chart_of_accounts::subAccountSet> for LedgerAccountSetDetails {
    fn from(account_set: chart_of_accounts::subAccountSet) -> Self {
        LedgerAccountSetDetails {
            id: account_set.account_set_details.account_set_id.into(),
            name: account_set.account_set_details.name,
            normal_balance_type: account_set.account_set_details.normal_balance_type.into(),
            page_info: account_set.account_set_details.members.page_info.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LedgerAccountSetSubAccount {
    Account(LedgerAccountDetails),
    AccountSet(LedgerAccountSetDetails),
}

impl From<chart_of_accounts::account> for Vec<LedgerAccountSetSubAccount> {
    fn from(members: chart_of_accounts::account) -> Self {
        members
            .edges
            .into_iter()
            .map(|e| match e.node {
                chart_of_accounts::AccountEdgesNode::Account(node) => {
                    LedgerAccountSetSubAccount::Account(LedgerAccountDetails::from(node))
                }
                chart_of_accounts::AccountEdgesNode::AccountSet(node) => {
                    LedgerAccountSetSubAccount::AccountSet(LedgerAccountSetDetails::from(node))
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PaginatedLedgerAccountSetSubAccount {
    pub cursor: String,
    pub value: LedgerAccountSetSubAccount,
}

#[derive(Debug, Clone)]
pub struct LedgerAccountSetSubAccounts {
    pub page_info: ConnectionCreationPageInfo,
    pub members: Vec<PaginatedLedgerAccountSetSubAccount>,
}

impl From<chart_of_accounts::subAccount> for LedgerAccountSetSubAccounts {
    fn from(sub_account: chart_of_accounts::subAccount) -> Self {
        let members = sub_account
            .edges
            .into_iter()
            .map(|e| match e.node {
                chart_of_accounts::SubAccountEdgesNode::Account(node) => {
                    PaginatedLedgerAccountSetSubAccount {
                        cursor: e.cursor,
                        value: LedgerAccountSetSubAccount::Account(LedgerAccountDetails::from(
                            node,
                        )),
                    }
                }
                chart_of_accounts::SubAccountEdgesNode::AccountSet(node) => {
                    PaginatedLedgerAccountSetSubAccount {
                        cursor: e.cursor,
                        value: LedgerAccountSetSubAccount::AccountSet(
                            LedgerAccountSetDetails::from(node),
                        ),
                    }
                }
            })
            .collect();

        LedgerAccountSetSubAccounts {
            page_info: ConnectionCreationPageInfo {
                has_next_page: sub_account.page_info.has_next_page,
                end_cursor: sub_account.page_info.end_cursor,
            },
            members,
        }
    }
}

impl From<account_set_and_sub_accounts::subAccount> for LedgerAccountSetSubAccounts {
    fn from(sub_account: account_set_and_sub_accounts::subAccount) -> Self {
        let members = sub_account
            .edges
            .into_iter()
            .map(|e| match e.node {
                account_set_and_sub_accounts::SubAccountEdgesNode::Account(node) => {
                    PaginatedLedgerAccountSetSubAccount {
                        cursor: e.cursor,
                        value: LedgerAccountSetSubAccount::Account(LedgerAccountDetails::from(
                            node,
                        )),
                    }
                }
                account_set_and_sub_accounts::SubAccountEdgesNode::AccountSet(node) => {
                    PaginatedLedgerAccountSetSubAccount {
                        cursor: e.cursor,
                        value: LedgerAccountSetSubAccount::AccountSet(
                            LedgerAccountSetDetails::from(node),
                        ),
                    }
                }
            })
            .collect();

        LedgerAccountSetSubAccounts {
            page_info: ConnectionCreationPageInfo {
                has_next_page: sub_account.page_info.has_next_page,
                end_cursor: sub_account.page_info.end_cursor,
            },
            members,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedgerAccountSetAndSubAccounts {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub sub_accounts: LedgerAccountSetSubAccounts,
}

impl From<account_set_and_sub_accounts::AccountSetAndSubAccountsAccountSet>
    for LedgerAccountSetAndSubAccounts
{
    fn from(account_set: account_set_and_sub_accounts::AccountSetAndSubAccountsAccountSet) -> Self {
        LedgerAccountSetAndSubAccounts {
            id: account_set.account_set_details.account_set_id.into(),
            name: account_set.account_set_details.name,
            normal_balance_type: account_set.account_set_details.normal_balance_type.into(),
            sub_accounts: account_set.sub_accounts.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedgerChartOfAccountsCategory {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub category_accounts: Vec<LedgerAccountSetSubAccount>,
}

impl From<chart_of_accounts::ChartOfAccountsAccountSetCategoriesEdgesNodeOnAccountSet>
    for LedgerChartOfAccountsCategory
{
    fn from(
        account_set: chart_of_accounts::ChartOfAccountsAccountSetCategoriesEdgesNodeOnAccountSet,
    ) -> Self {
        LedgerChartOfAccountsCategory {
            id: account_set.account_set_details.account_set_id.into(),
            name: account_set.account_set_details.name,
            normal_balance_type: account_set.account_set_details.normal_balance_type.into(),
            category_accounts: account_set.accounts.into(),
        }
    }
}

impl From<chart_of_accounts::ChartOfAccountsAccountSetCategories>
    for Vec<LedgerChartOfAccountsCategory>
{
    fn from(members: chart_of_accounts::ChartOfAccountsAccountSetCategories) -> Self {
        members
            .edges
            .into_iter()
            .filter_map(|e| match e.node {
                chart_of_accounts::ChartOfAccountsAccountSetCategoriesEdgesNode::Account(_) => None,
                chart_of_accounts::ChartOfAccountsAccountSetCategoriesEdgesNode::AccountSet(
                    node,
                ) => Some(LedgerChartOfAccountsCategory::from(node)),
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct LedgerChartOfAccounts {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub categories: Vec<LedgerChartOfAccountsCategory>,
}

impl From<chart_of_accounts::ChartOfAccountsAccountSet> for LedgerChartOfAccounts {
    fn from(account_set: chart_of_accounts::ChartOfAccountsAccountSet) -> Self {
        LedgerChartOfAccounts {
            name: account_set.account_set_details.name,
            normal_balance_type: account_set.account_set_details.normal_balance_type.into(),
            categories: account_set.categories.into(),
        }
    }
}
