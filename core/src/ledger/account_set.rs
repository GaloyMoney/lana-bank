use crate::primitives::{
    LedgerAccountId, LedgerAccountSetId, LedgerAccountSetMemberType, LedgerDebitOrCredit,
};

use super::{account::*, cala::graphql::*};

#[derive(Debug, Clone)]
pub struct LedgerAccountSetBalance {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
}

impl From<trial_balance::TrialBalanceAccountSetMembersEdgesNodeOnAccountSet>
    for LedgerAccountSetBalance
{
    fn from(node: trial_balance::TrialBalanceAccountSetMembersEdgesNodeOnAccountSet) -> Self {
        LedgerAccountSetBalance {
            name: node.name,
            normal_balance_type: node.normal_balance_type.into(),
            balance: LedgerAccountBalancesByCurrency {
                btc: node.btc_balances.map_or_else(
                    LayeredBtcAccountBalances::default,
                    LayeredBtcAccountBalances::from,
                ),
                usd: node.usd_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
                usdt: node.usdt_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum LedgerAccountSetMemberBalance {
    LedgerAccountBalance(LedgerAccountBalance),
    LedgerAccountSetBalance(LedgerAccountSetBalance),
}

pub struct LedgerAccountSetAndMemberBalances {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub member_balances: Vec<LedgerAccountSetMemberBalance>,
}

impl From<trial_balance::TrialBalanceAccountSet> for LedgerAccountSetAndMemberBalances {
    fn from(account_set: trial_balance::TrialBalanceAccountSet) -> Self {
        let member_balances: Vec<LedgerAccountSetMemberBalance> = account_set
            .members
            .edges
            .iter()
            .map(|e| match &e.node {
                trial_balance::TrialBalanceAccountSetMembersEdgesNode::Account(node) => {
                    LedgerAccountSetMemberBalance::LedgerAccountBalance(LedgerAccountBalance::from(
                        node.clone(),
                    ))
                }
                trial_balance::TrialBalanceAccountSetMembersEdgesNode::AccountSet(node) => {
                    LedgerAccountSetMemberBalance::LedgerAccountSetBalance(
                        LedgerAccountSetBalance::from(node.clone()),
                    )
                }
            })
            .collect();

        Self {
            name: account_set.name,
            normal_balance_type: account_set.normal_balance_type.into(),
            balance: LedgerAccountBalancesByCurrency {
                btc: account_set.btc_balances.map_or_else(
                    LayeredBtcAccountBalances::default,
                    LayeredBtcAccountBalances::from,
                ),
                usd: account_set.usd_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
                usdt: account_set.usdt_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
            },
            member_balances,
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
pub struct LedgerChartOfAccountsAccount {
    pub id: LedgerAccountId,
    pub code: String,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
}

impl From<chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccount>
    for LedgerChartOfAccountsAccount
{
    fn from(
        account: chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccount,
    ) -> Self {
        LedgerChartOfAccountsAccount {
            id: account.account_id.into(),
            code: account.code,
            name: account.name,
            normal_balance_type: account.normal_balance_type.into(),
        }
    }
}

impl From<chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccountSetMembersEdgesNodeOnAccount>
    for LedgerChartOfAccountsAccount
{
    fn from(
        account: chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccountSetMembersEdgesNodeOnAccount,
    ) -> Self {
        LedgerChartOfAccountsAccount {
            id: account.account_id.into(),
            code: account.code,
            name: account.name,
            normal_balance_type: account.normal_balance_type.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedgerChartOfAccountsSubAccountSet {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
}

impl From<chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccountSetMembersEdgesNodeOnAccountSet> for LedgerChartOfAccountsSubAccountSet {
    fn from(account_set: chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccountSetMembersEdgesNodeOnAccountSet) -> Self {
        LedgerChartOfAccountsSubAccountSet{
            id: account_set.account_set_id.into(),
            name: account_set.name,
            normal_balance_type: account_set.normal_balance_type.into()
        }
    }
}

#[derive(Debug, Clone)]
pub enum LedgerChartOfAccountsSubMember {
    Account(LedgerChartOfAccountsAccount),
    AccountSet(LedgerChartOfAccountsSubAccountSet),
}

impl
    From<chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccountSetMembers>
    for Vec<LedgerChartOfAccountsSubMember>
{
    fn from(
        members: chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccountSetMembers,
    ) -> Self {
        members
            .edges
            .iter()
            .map(|e| match &e.node {
                chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccountSetMembersEdgesNode::Account(node) => {
                    LedgerChartOfAccountsSubMember::Account(LedgerChartOfAccountsAccount::from(
                        node.clone(),
                    ))
                }
                chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccountSetMembersEdgesNode::AccountSet(node) => {
                    LedgerChartOfAccountsSubMember::AccountSet(
                        LedgerChartOfAccountsSubAccountSet::from(node.clone()),
                    )
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct LedgerChartOfAccountsAccountSet {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub members: Vec<LedgerChartOfAccountsSubMember>,
}

impl From<chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccountSet>
    for LedgerChartOfAccountsAccountSet
{
    fn from(
        account_set: chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNodeOnAccountSet,
    ) -> Self {
        LedgerChartOfAccountsAccountSet {
            id: account_set.account_set_id.into(),
            name: account_set.name,
            normal_balance_type: account_set.normal_balance_type.into(),
            members: account_set.members.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LedgerChartOfAccountsMember {
    Account(LedgerChartOfAccountsAccount),
    AccountSet(LedgerChartOfAccountsAccountSet),
}

impl From<chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembers>
    for Vec<LedgerChartOfAccountsMember>
{
    fn from(members: chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembers) -> Self {
        members
            .edges
            .iter()
            .map(|e| match &e.node {
                chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNode::Account(node) => {
                    LedgerChartOfAccountsMember::Account(LedgerChartOfAccountsAccount::from(
                        node.clone(),
                    ))
                }
                chart_of_accounts_group::ChartOfAccountsGroupAccountSetMembersEdgesNode::AccountSet(node) => {
                    LedgerChartOfAccountsMember::AccountSet(
                        LedgerChartOfAccountsAccountSet::from(node.clone()),
                    )
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct LedgerChartOfAccountsGroup {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub members: Vec<LedgerChartOfAccountsMember>,
}

impl From<chart_of_accounts_group::ChartOfAccountsGroupAccountSet> for LedgerChartOfAccountsGroup {
    fn from(account_set: chart_of_accounts_group::ChartOfAccountsGroupAccountSet) -> Self {
        LedgerChartOfAccountsGroup {
            name: account_set.name,
            normal_balance_type: account_set.normal_balance_type.into(),
            members: account_set.members.into(),
        }
    }
}
