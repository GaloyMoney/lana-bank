use async_graphql::*;

use lana_app::chart_of_accounts::chart::*;

#[derive(SimpleObject)]
pub struct ChartOfAccounts {
    name: String,
    categories: ChartCategories,
}

impl From<ChartOfAccountsProjection> for ChartOfAccounts {
    fn from(projection: ChartOfAccountsProjection) -> Self {
        ChartOfAccounts {
            name: projection.name,
            categories: ChartCategories {
                assets: ChartCategory {
                    name: projection.assets.name,
                    account_code: projection.assets.encoded_path,
                    control_accounts: projection
                        .assets
                        .children
                        .into_iter()
                        .map(ChartControlAccount::from)
                        .collect(),
                },
                liabilities: ChartCategory {
                    name: projection.liabilities.name,
                    account_code: projection.liabilities.encoded_path,
                    control_accounts: projection
                        .liabilities
                        .children
                        .into_iter()
                        .map(ChartControlAccount::from)
                        .collect(),
                },
                equity: ChartCategory {
                    name: projection.equity.name,
                    account_code: projection.equity.encoded_path,
                    control_accounts: projection
                        .equity
                        .children
                        .into_iter()
                        .map(ChartControlAccount::from)
                        .collect(),
                },
                revenues: ChartCategory {
                    name: projection.revenues.name,
                    account_code: projection.revenues.encoded_path,
                    control_accounts: projection
                        .revenues
                        .children
                        .into_iter()
                        .map(ChartControlAccount::from)
                        .collect(),
                },
                expenses: ChartCategory {
                    name: projection.expenses.name,
                    account_code: projection.expenses.encoded_path,
                    control_accounts: projection
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

impl From<ControlAccountProjection> for ChartControlAccount {
    fn from(projection: ControlAccountProjection) -> Self {
        ChartControlAccount {
            name: projection.name,
            account_code: projection.encoded_path,
            control_sub_accounts: projection
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

impl From<ControlSubAccountProjection> for ChartControlSubAccount {
    fn from(projection: ControlSubAccountProjection) -> Self {
        ChartControlSubAccount {
            name: projection.name,
            account_code: projection.encoded_path,
        }
    }
}
