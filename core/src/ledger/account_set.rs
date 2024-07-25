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

impl From<trial_balance::AccountsEdgesNodeOnAccountSet> for LedgerAccountSetWithBalance {
    fn from(node: trial_balance::AccountsEdgesNodeOnAccountSet) -> Self {
        LedgerAccountSetWithBalance {
            id: node.account_set_details.account_set_id.into(),
            name: node.account_set_details.name,
            normal_balance_type: node.account_set_details.normal_balance_type.into(),
            balance: node.account_set_balances.into(),
            has_sub_accounts: node
                .account_set_details
                .members
                .page_info
                .start_cursor
                .is_some(),
        }
    }
}

impl From<balance_sheet::CategoriesEdgesNodeOnAccountSet> for LedgerAccountSetWithBalance {
    fn from(node: balance_sheet::CategoriesEdgesNodeOnAccountSet) -> Self {
        LedgerAccountSetWithBalance {
            id: node.account_set_details.account_set_id.into(),
            name: node.account_set_details.name,
            normal_balance_type: node.account_set_details.normal_balance_type.into(),
            balance: node.account_set_balances.into(),
            has_sub_accounts: node
                .account_set_details
                .members
                .page_info
                .start_cursor
                .is_some(),
        }
    }
}

impl From<balance_sheet::AccountsEdgesNodeOnAccountSet> for LedgerAccountSetWithBalance {
    fn from(node: balance_sheet::AccountsEdgesNodeOnAccountSet) -> Self {
        LedgerAccountSetWithBalance {
            id: node.account_set_details.account_set_id.into(),
            name: node.account_set_details.name,
            normal_balance_type: node.account_set_details.normal_balance_type.into(),
            balance: node.account_set_balances.into(),
            has_sub_accounts: node
                .account_set_details
                .members
                .page_info
                .start_cursor
                .is_some(),
        }
    }
}

impl From<profit_and_loss_statement::CategoriesEdgesNodeOnAccountSet>
    for LedgerAccountSetWithBalance
{
    fn from(node: profit_and_loss_statement::CategoriesEdgesNodeOnAccountSet) -> Self {
        LedgerAccountSetWithBalance {
            id: node.account_set_details.account_set_id.into(),
            name: node.account_set_details.name,
            normal_balance_type: node.account_set_details.normal_balance_type.into(),
            balance: node.account_set_balances.into(),
            has_sub_accounts: node
                .account_set_details
                .members
                .page_info
                .start_cursor
                .is_some(),
        }
    }
}

impl From<profit_and_loss_statement::AccountsEdgesNodeOnAccountSet>
    for LedgerAccountSetWithBalance
{
    fn from(node: profit_and_loss_statement::AccountsEdgesNodeOnAccountSet) -> Self {
        LedgerAccountSetWithBalance {
            id: node.account_set_details.account_set_id.into(),
            name: node.account_set_details.name,
            normal_balance_type: node.account_set_details.normal_balance_type.into(),
            balance: node.account_set_balances.into(),
            has_sub_accounts: node
                .account_set_details
                .members
                .page_info
                .start_cursor
                .is_some(),
        }
    }
}

impl From<account_set_and_sub_accounts_with_balance::accountSetDetailsAndBalances>
    for LedgerAccountSetWithBalance
{
    fn from(
        account_set: account_set_and_sub_accounts_with_balance::accountSetDetailsAndBalances,
    ) -> Self {
        let account_set_details = account_set.account_set_details;
        LedgerAccountSetWithBalance {
            id: account_set_details.account_set_id.into(),
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            balance: account_set.account_set_balances.into(),
            has_sub_accounts: account_set_details.members.page_info.start_cursor.is_some(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LedgerAccountSetSubAccountWithBalance {
    Account(LedgerAccountWithBalance),
    AccountSet(LedgerAccountSetWithBalance),
}

impl From<trial_balance::TrialBalanceAccountSetAccounts>
    for Vec<LedgerAccountSetSubAccountWithBalance>
{
    fn from(members: trial_balance::TrialBalanceAccountSetAccounts) -> Self {
        members
            .edges
            .into_iter()
            .map(|e| match e.node {
                trial_balance::AccountsEdgesNode::Account(node) => {
                    LedgerAccountSetSubAccountWithBalance::Account(LedgerAccountWithBalance::from(
                        node,
                    ))
                }
                trial_balance::AccountsEdgesNode::AccountSet(node) => {
                    LedgerAccountSetSubAccountWithBalance::AccountSet(
                        LedgerAccountSetWithBalance::from(node),
                    )
                }
            })
            .collect()
    }
}

impl From<balance_sheet::categories> for Vec<LedgerAccountSetSubAccountWithBalance> {
    fn from(members: balance_sheet::categories) -> Self {
        members
            .edges
            .into_iter()
            .map(|e| match e.node {
                balance_sheet::CategoriesEdgesNode::Account(node) => {
                    LedgerAccountSetSubAccountWithBalance::Account(LedgerAccountWithBalance::from(
                        node,
                    ))
                }
                balance_sheet::CategoriesEdgesNode::AccountSet(node) => {
                    LedgerAccountSetSubAccountWithBalance::AccountSet(
                        LedgerAccountSetWithBalance::from(node),
                    )
                }
            })
            .collect()
    }
}

impl From<balance_sheet::accounts> for Vec<LedgerAccountSetSubAccountWithBalance> {
    fn from(members: balance_sheet::accounts) -> Self {
        members
            .edges
            .into_iter()
            .map(|e| match e.node {
                balance_sheet::AccountsEdgesNode::Account(node) => {
                    LedgerAccountSetSubAccountWithBalance::Account(LedgerAccountWithBalance::from(
                        node,
                    ))
                }
                balance_sheet::AccountsEdgesNode::AccountSet(node) => {
                    LedgerAccountSetSubAccountWithBalance::AccountSet(
                        LedgerAccountSetWithBalance::from(node),
                    )
                }
            })
            .collect()
    }
}

impl From<profit_and_loss_statement::categories> for Vec<LedgerAccountSetSubAccountWithBalance> {
    fn from(members: profit_and_loss_statement::categories) -> Self {
        members
            .edges
            .into_iter()
            .map(|e| match e.node {
                profit_and_loss_statement::CategoriesEdgesNode::Account(node) => {
                    LedgerAccountSetSubAccountWithBalance::Account(LedgerAccountWithBalance::from(
                        node,
                    ))
                }
                profit_and_loss_statement::CategoriesEdgesNode::AccountSet(node) => {
                    LedgerAccountSetSubAccountWithBalance::AccountSet(
                        LedgerAccountSetWithBalance::from(node),
                    )
                }
            })
            .collect()
    }
}

impl From<profit_and_loss_statement::accounts> for Vec<LedgerAccountSetSubAccountWithBalance> {
    fn from(members: profit_and_loss_statement::accounts) -> Self {
        members
            .edges
            .into_iter()
            .map(|e| match e.node {
                profit_and_loss_statement::AccountsEdgesNode::Account(node) => {
                    LedgerAccountSetSubAccountWithBalance::Account(LedgerAccountWithBalance::from(
                        node,
                    ))
                }
                profit_and_loss_statement::AccountsEdgesNode::AccountSet(node) => {
                    LedgerAccountSetSubAccountWithBalance::AccountSet(
                        LedgerAccountSetWithBalance::from(node),
                    )
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PaginatedLedgerAccountSetSubAccountWithBalance {
    pub cursor: String,
    pub value: LedgerAccountSetSubAccountWithBalance,
}

#[derive(Debug, Clone)]
pub struct LedgerAccountSetSubAccountsWithBalance {
    pub page_info: ConnectionCreationPageInfo,
    pub members: Vec<PaginatedLedgerAccountSetSubAccountWithBalance>,
}

impl From<account_set_and_sub_accounts_with_balance::subAccountWithBalances>
    for LedgerAccountSetSubAccountsWithBalance
{
    fn from(
        sub_account: account_set_and_sub_accounts_with_balance::subAccountWithBalances,
    ) -> Self {
        let members = sub_account
            .edges
            .into_iter()
            .map(|e| match e.node {
                account_set_and_sub_accounts_with_balance::SubAccountWithBalancesEdgesNode::Account(node) => {
                    PaginatedLedgerAccountSetSubAccountWithBalance {
                        cursor: e.cursor,
                        value: LedgerAccountSetSubAccountWithBalance::Account(
                            LedgerAccountWithBalance::from(node),
                        ),
                    }
                }
                account_set_and_sub_accounts_with_balance::SubAccountWithBalancesEdgesNode::AccountSet(
                    node,
                ) => PaginatedLedgerAccountSetSubAccountWithBalance {
                    cursor: e.cursor,
                    value: LedgerAccountSetSubAccountWithBalance::AccountSet(
                        LedgerAccountSetWithBalance::from(node),
                    ),
                },
            })
            .collect();

        LedgerAccountSetSubAccountsWithBalance {
            page_info: ConnectionCreationPageInfo {
                has_next_page: sub_account.page_info.has_next_page,
                end_cursor: sub_account.page_info.end_cursor,
            },
            members,
        }
    }
}

pub struct LedgerAccountSetAndSubAccountsWithBalance {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub sub_accounts: LedgerAccountSetSubAccountsWithBalance,
}

impl From<account_set_and_sub_accounts_with_balance::AccountSetAndSubAccountsWithBalanceAccountSet>
    for LedgerAccountSetAndSubAccountsWithBalance
{
    fn from(
        account_set: account_set_and_sub_accounts_with_balance::AccountSetAndSubAccountsWithBalanceAccountSet,
    ) -> Self {
        let account_set_details = account_set.account_set_details;
        LedgerAccountSetAndSubAccountsWithBalance {
            id: account_set_details.account_set_id.into(),
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            balance: account_set.account_set_balances.into(),
            sub_accounts: account_set.sub_accounts.into(),
        }
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

impl From<chart_of_accounts::accounts> for Vec<LedgerAccountSetSubAccount> {
    fn from(members: chart_of_accounts::accounts) -> Self {
        members
            .edges
            .into_iter()
            .map(|e| match e.node {
                chart_of_accounts::AccountsEdgesNode::Account(node) => {
                    LedgerAccountSetSubAccount::Account(LedgerAccountDetails::from(node))
                }
                chart_of_accounts::AccountsEdgesNode::AccountSet(node) => {
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
        let account_set_details = account_set.account_set_details;
        LedgerAccountSetAndSubAccounts {
            id: account_set_details.account_set_id.into(),
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
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

impl From<chart_of_accounts::CategoriesEdgesNodeOnAccountSet> for LedgerChartOfAccountsCategory {
    fn from(account_set: chart_of_accounts::CategoriesEdgesNodeOnAccountSet) -> Self {
        let account_set_details = account_set.account_set_details;
        LedgerChartOfAccountsCategory {
            id: account_set_details.account_set_id.into(),
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
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
                chart_of_accounts::CategoriesEdgesNode::Account(_) => None,
                chart_of_accounts::CategoriesEdgesNode::AccountSet(node) => {
                    Some(LedgerChartOfAccountsCategory::from(node))
                }
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

#[derive(Debug, Clone)]
pub struct LedgerTrialBalance {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub accounts: Vec<LedgerAccountSetSubAccountWithBalance>,
}

impl From<trial_balance::TrialBalanceAccountSet> for LedgerTrialBalance {
    fn from(account_set: trial_balance::TrialBalanceAccountSet) -> Self {
        LedgerTrialBalance {
            name: account_set.name,
            normal_balance_type: account_set.normal_balance_type.into(),
            balance: account_set.account_set_balances.into(),
            accounts: account_set.accounts.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedgerStatementCategory {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub accounts: Vec<LedgerAccountSetSubAccountWithBalance>,
}

impl From<balance_sheet::CategoriesEdgesNodeOnAccountSet> for LedgerStatementCategory {
    fn from(account_set: balance_sheet::CategoriesEdgesNodeOnAccountSet) -> Self {
        let account_set_details = account_set.account_set_details;
        LedgerStatementCategory {
            id: account_set_details.account_set_id.into(),
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            balance: account_set.account_set_balances.into(),
            accounts: account_set.accounts.into(),
        }
    }
}

impl From<balance_sheet::BalanceSheetAccountSetCategories> for Vec<LedgerStatementCategory> {
    fn from(members: balance_sheet::BalanceSheetAccountSetCategories) -> Self {
        members
            .edges
            .into_iter()
            .filter_map(|e| match e.node {
                balance_sheet::CategoriesEdgesNode::Account(_) => None,
                balance_sheet::CategoriesEdgesNode::AccountSet(node) => {
                    Some(LedgerStatementCategory::from(node))
                }
            })
            .collect()
    }
}

impl From<profit_and_loss_statement::CategoriesEdgesNodeOnAccountSet> for LedgerStatementCategory {
    fn from(account_set: profit_and_loss_statement::CategoriesEdgesNodeOnAccountSet) -> Self {
        let account_set_details = account_set.account_set_details;
        LedgerStatementCategory {
            id: account_set_details.account_set_id.into(),
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            balance: account_set.account_set_balances.into(),
            accounts: account_set.accounts.into(),
        }
    }
}

impl From<profit_and_loss_statement::ProfitAndLossStatementAccountSetCategories>
    for Vec<LedgerStatementCategory>
{
    fn from(
        members: profit_and_loss_statement::ProfitAndLossStatementAccountSetCategories,
    ) -> Self {
        members
            .edges
            .into_iter()
            .filter_map(|e| match e.node {
                profit_and_loss_statement::CategoriesEdgesNode::Account(_) => None,
                profit_and_loss_statement::CategoriesEdgesNode::AccountSet(node) => {
                    Some(LedgerStatementCategory::from(node))
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct LedgerBalanceSheet {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub categories: Vec<LedgerStatementCategory>,
}

impl From<balance_sheet::BalanceSheetAccountSet> for LedgerBalanceSheet {
    fn from(account_set: balance_sheet::BalanceSheetAccountSet) -> Self {
        let account_set_details = account_set.account_set_details;
        LedgerBalanceSheet {
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            balance: account_set.account_set_balances.into(),
            categories: account_set.categories.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedgerProfitAndLossStatement {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub categories: Vec<LedgerStatementCategory>,
}

impl From<profit_and_loss_statement::ProfitAndLossStatementAccountSet>
    for LedgerProfitAndLossStatement
{
    fn from(account_set: profit_and_loss_statement::ProfitAndLossStatementAccountSet) -> Self {
        let account_set_details = account_set.account_set_details;
        LedgerProfitAndLossStatement {
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            balance: account_set.account_set_balances.into(),
            categories: account_set.categories.into(),
        }
    }
}
