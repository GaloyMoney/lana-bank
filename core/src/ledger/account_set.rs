use crate::primitives::{LedgerAccountSetId, LedgerDebitOrCredit};

use super::{account::*, cala::graphql::*, error::*};

#[derive(Debug, Clone)]
pub struct LedgerSubAccountCursor {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct PageExistsPageInfo {
    pub start_cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LedgerAccountSetWithBalance {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub page_info: PageExistsPageInfo,
}

#[derive(Debug, Clone)]
pub enum LedgerAccountSetSubAccountWithBalance {
    Account(LedgerAccountWithBalance),
    AccountSet(LedgerAccountSetWithBalance),
}

macro_rules! impl_from_account_set_details_and_balances {
    ($($module:ident),+)  => {
        $(
            impl From<$module::AccountSetDetailsMembersPageInfo> for PageExistsPageInfo {
                fn from(page_info: $module::AccountSetDetailsMembersPageInfo) -> Self {
                    PageExistsPageInfo {
                        start_cursor: page_info.start_cursor,
                    }
                }
            }

            impl TryFrom<$module::accountSetDetailsAndBalances> for LedgerAccountSetWithBalance {
                type Error = LedgerError;

                fn try_from(account_set: $module::accountSetDetailsAndBalances) -> Result<Self, Self::Error> {
                    let account_set_details = account_set.account_set_details;
                    Ok(LedgerAccountSetWithBalance {
                        id: account_set_details.account_set_id.into(),
                        name: account_set_details.name,
                        normal_balance_type: account_set_details.normal_balance_type.into(),
                        balance: account_set.account_set_balances.try_into()?,
                        page_info: account_set_details.members.page_info.into(),
                    })
                }
            }
        )+
    };
}

macro_rules! impl_from_accounts_with_balances {
    ($($module:ident),+)  => {
        $(
            impl TryFrom<$module::accountsWithBalances> for Vec<LedgerAccountSetSubAccountWithBalance> {
                type Error = LedgerError;

                fn try_from(members: $module::accountsWithBalances) -> Result<Self, Self::Error> {
                    members
                        .edges
                        .into_iter()
                        .map(|e| match e.node {
                            $module::AccountsWithBalancesEdgesNode::Account(node) => {
                                LedgerAccountWithBalance::try_from(node)
                                    .map(LedgerAccountSetSubAccountWithBalance::Account)
                            }
                            $module::AccountsWithBalancesEdgesNode::AccountSet(node) => {
                                LedgerAccountSetWithBalance::try_from(node)
                                    .map(LedgerAccountSetSubAccountWithBalance::AccountSet)
                            }
                        })
                        .collect()
                }
            }
        )+
    };
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

impl TryFrom<account_set_and_sub_accounts_with_balance::subAccountsWithBalances>
    for LedgerAccountSetSubAccountsWithBalance
{
    type Error = LedgerError;

    fn try_from(
        sub_account: account_set_and_sub_accounts_with_balance::subAccountsWithBalances,
    ) -> Result<Self, Self::Error> {
        let members = sub_account
            .edges
            .into_iter()
            .map(|e| -> Result<PaginatedLedgerAccountSetSubAccountWithBalance, Self::Error> {
                let value = match e.node {
                    account_set_and_sub_accounts_with_balance::SubAccountsWithBalancesEdgesNode::Account(node) => {
                        LedgerAccountWithBalance::try_from(node)
                            .map(LedgerAccountSetSubAccountWithBalance::Account)
                    }
                    account_set_and_sub_accounts_with_balance::SubAccountsWithBalancesEdgesNode::AccountSet(
                        node,
                    ) => {
                        LedgerAccountSetWithBalance::try_from(node)
                            .map(LedgerAccountSetSubAccountWithBalance::AccountSet)
                    }
                }?;

                Ok(PaginatedLedgerAccountSetSubAccountWithBalance {
                    cursor: e.cursor,
                    value,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(LedgerAccountSetSubAccountsWithBalance {
            page_info: ConnectionCreationPageInfo {
                has_next_page: sub_account.page_info.has_next_page,
                end_cursor: sub_account.page_info.end_cursor,
            },
            members,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionCreationPageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LedgerStatementCategoryWithBalance {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub accounts: Vec<LedgerAccountSetSubAccountWithBalance>,
}

macro_rules! impl_from_category_with_balances {
    ($($module:ident),+)  => {
        $(
            impl TryFrom<$module::categoryAccountSetWithBalances> for LedgerStatementCategoryWithBalance {
                type Error = LedgerError;

                fn try_from(account_set: $module::categoryAccountSetWithBalances) -> Result<Self, Self::Error> {
                    let account_set_details = account_set
                        .account_set_details_and_balances
                        .account_set_details;
                    Ok(LedgerStatementCategoryWithBalance {
                        id: account_set_details.account_set_id.into(),
                        name: account_set_details.name,
                        normal_balance_type: account_set_details.normal_balance_type.into(),
                        balance: account_set
                            .account_set_details_and_balances
                            .account_set_balances
                            .try_into()?,
                        accounts: account_set.accounts.try_into()?,
                    })
                }
            }

            impl TryFrom<$module::categoriesWithBalances> for Vec<LedgerStatementCategoryWithBalance> {
                type Error = LedgerError;

                fn try_from(members: $module::categoriesWithBalances) -> Result<Self, Self::Error> {
                    members
                        .edges
                        .into_iter()
                        .filter_map(|e| match e.node {
                            $module::CategoriesWithBalancesEdgesNode::Account(_) => None,
                            $module::CategoriesWithBalancesEdgesNode::AccountSet(node) => {
                                Some(LedgerStatementCategoryWithBalance::try_from(node))
                            }
                        })
                        .collect()
                }
            }
        )+
    };
}

impl_from_account_set_details_and_balances!(
    chart_of_accounts,
    account_set_and_sub_accounts_with_balance,
    trial_balance,
    balance_sheet,
    profit_and_loss_statement
);

impl_from_accounts_with_balances!(
    chart_of_accounts,
    trial_balance,
    balance_sheet,
    profit_and_loss_statement
);

impl_from_category_with_balances!(chart_of_accounts, balance_sheet, profit_and_loss_statement);

// QUERIES TO EXPLORE REPORTS

pub struct LedgerAccountSetAndSubAccountsWithBalance {
    pub id: LedgerAccountSetId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub sub_accounts: LedgerAccountSetSubAccountsWithBalance,
}

impl
    TryFrom<
        account_set_and_sub_accounts_with_balance::AccountSetAndSubAccountsWithBalanceAccountSet,
    > for LedgerAccountSetAndSubAccountsWithBalance
{
    type Error = LedgerError;

    fn try_from(
        account_set: account_set_and_sub_accounts_with_balance::AccountSetAndSubAccountsWithBalanceAccountSet,
    ) -> Result<Self, Self::Error> {
        let account_set_details = account_set
            .account_set_details_and_balances
            .account_set_details;

        Ok(LedgerAccountSetAndSubAccountsWithBalance {
            id: account_set_details.account_set_id.into(),
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            balance: account_set
                .account_set_details_and_balances
                .account_set_balances
                .try_into()?,
            sub_accounts: account_set.sub_accounts.try_into()?,
        })
    }
}

// TOP-LEVEL REPORTS

#[derive(Debug, Clone)]
pub struct LedgerChartOfAccounts {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub categories: Vec<LedgerStatementCategoryWithBalance>,
}

impl TryFrom<chart_of_accounts::ChartOfAccountsAccountSet> for LedgerChartOfAccounts {
    type Error = LedgerError;

    fn try_from(
        account_set: chart_of_accounts::ChartOfAccountsAccountSet,
    ) -> Result<Self, Self::Error> {
        let account_set_details = account_set
            .account_set_details_and_balances
            .account_set_details;

        Ok(LedgerChartOfAccounts {
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            categories: account_set.categories.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LedgerTrialBalance {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub accounts: Vec<LedgerAccountSetSubAccountWithBalance>,
}

impl TryFrom<trial_balance::TrialBalanceAccountSet> for LedgerTrialBalance {
    type Error = LedgerError;

    fn try_from(account_set: trial_balance::TrialBalanceAccountSet) -> Result<Self, Self::Error> {
        Ok(LedgerTrialBalance {
            name: account_set.name,
            normal_balance_type: account_set.normal_balance_type.into(),
            balance: account_set.account_set_balances.try_into()?,
            accounts: account_set.accounts.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LedgerBalanceSheet {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub categories: Vec<LedgerStatementCategoryWithBalance>,
}

impl TryFrom<balance_sheet::BalanceSheetAccountSet> for LedgerBalanceSheet {
    type Error = LedgerError;

    fn try_from(account_set: balance_sheet::BalanceSheetAccountSet) -> Result<Self, Self::Error> {
        let account_set_details = account_set
            .account_set_details_and_balances
            .account_set_details;

        Ok(LedgerBalanceSheet {
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            balance: account_set
                .account_set_details_and_balances
                .account_set_balances
                .try_into()?,
            categories: account_set.categories.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LedgerProfitAndLossStatement {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
    pub categories: Vec<LedgerStatementCategoryWithBalance>,
}

impl TryFrom<profit_and_loss_statement::ProfitAndLossStatementAccountSet>
    for LedgerProfitAndLossStatement
{
    type Error = LedgerError;

    fn try_from(
        account_set: profit_and_loss_statement::ProfitAndLossStatementAccountSet,
    ) -> Result<Self, Self::Error> {
        let account_set_details = account_set
            .account_set_details_and_balances
            .account_set_details;

        Ok(LedgerProfitAndLossStatement {
            name: account_set_details.name,
            normal_balance_type: account_set_details.normal_balance_type.into(),
            balance: account_set
                .account_set_details_and_balances
                .account_set_balances
                .try_into()?,
            categories: account_set.categories.try_into()?,
        })
    }
}
