use async_graphql::*;

use lana_app::chart_of_accounts::tree::*;

#[derive(SimpleObject)]
pub struct ChartOfAccounts {
    name: String,
    categories: ChartCategories,
}

impl From<ChartTree> for ChartOfAccounts {
    fn from(tree: ChartTree) -> Self {
        ChartOfAccounts {
            name: tree.name,
            categories: ChartCategories {
                assets: ChartCategory {
                    name: tree.assets.name,
                    account_code: tree.assets.encoded_path,
                    control_accounts: tree
                        .assets
                        .children
                        .into_iter()
                        .map(ChartControlAccount::from)
                        .collect(),
                },
                liabilities: ChartCategory {
                    name: tree.liabilities.name,
                    account_code: tree.liabilities.encoded_path,
                    control_accounts: tree
                        .liabilities
                        .children
                        .into_iter()
                        .map(ChartControlAccount::from)
                        .collect(),
                },
                equity: ChartCategory {
                    name: tree.equity.name,
                    account_code: tree.equity.encoded_path,
                    control_accounts: tree
                        .equity
                        .children
                        .into_iter()
                        .map(ChartControlAccount::from)
                        .collect(),
                },
                revenues: ChartCategory {
                    name: tree.revenues.name,
                    account_code: tree.revenues.encoded_path,
                    control_accounts: tree
                        .revenues
                        .children
                        .into_iter()
                        .map(ChartControlAccount::from)
                        .collect(),
                },
                expenses: ChartCategory {
                    name: tree.expenses.name,
                    account_code: tree.expenses.encoded_path,
                    control_accounts: tree
                        .expenses
                        .children
                        .into_iter()
                        .map(ChartControlAccount::from)
                        .collect(),
                },
            },
        }
    }
}

#[derive(SimpleObject)]
pub struct ChartCategories {
    assets: ChartCategory,
    liabilities: ChartCategory,
    equity: ChartCategory,
    revenues: ChartCategory,
    expenses: ChartCategory,
}

#[derive(SimpleObject)]
pub struct ChartCategory {
    name: String,
    account_code: String,
    control_accounts: Vec<ChartControlAccount>,
}

#[derive(SimpleObject)]
pub struct ChartControlAccount {
    name: String,
    account_code: String,
    control_sub_accounts: Vec<ChartControlSubAccount>,
}

impl From<ChartTreeControlAccount> for ChartControlAccount {
    fn from(tree: ChartTreeControlAccount) -> Self {
        ChartControlAccount {
            name: tree.name,
            account_code: tree.encoded_path,
            control_sub_accounts: tree
                .children
                .into_iter()
                .map(ChartControlSubAccount::from)
                .collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct ChartControlSubAccount {
    name: String,
    account_code: String,
}

impl From<ChartTreeControlSubAccount> for ChartControlSubAccount {
    fn from(tree: ChartTreeControlSubAccount) -> Self {
        ChartControlSubAccount {
            name: tree.name,
            account_code: tree.encoded_path,
        }
    }
}
