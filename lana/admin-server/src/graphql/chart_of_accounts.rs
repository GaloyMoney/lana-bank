use async_graphql::*;

use crate::primitives::*;

use lana_app::chart_of_accounts::Chart as DomainChart;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct ChartOfAccounts {
    id: ID,
    name: String,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainChart>,
}

impl From<DomainChart> for ChartOfAccounts {
    fn from(chart: DomainChart) -> Self {
        ChartOfAccounts {
            id: chart.id.to_global_id(),
            name: chart.name.to_string(),

            entity: Arc::new(chart),
        }
    }
}

#[ComplexObject]
impl ChartOfAccounts {
    async fn categories(&self) -> ChartCategories {
        let tree = self.entity.chart();
        ChartCategories {
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
pub struct ChartControlAccountDetails {
    name: String,
    account_code: String,
}

impl From<lana_app::chart_of_accounts::ControlAccountDetails> for ChartControlAccountDetails {
    fn from(details: lana_app::chart_of_accounts::ControlAccountDetails) -> Self {
        Self {
            name: details.name,
            account_code: details.path.to_string(),
        }
    }
}

#[derive(SimpleObject)]
pub struct ChartControlAccount {
    name: String,
    account_code: String,
    control_sub_accounts: Vec<ChartControlSubAccount>,
}

impl From<lana_app::chart_of_accounts::tree::ChartTreeControlAccount> for ChartControlAccount {
    fn from(tree: lana_app::chart_of_accounts::tree::ChartTreeControlAccount) -> Self {
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

impl From<lana_app::chart_of_accounts::tree::ChartTreeControlSubAccount>
    for ChartControlSubAccount
{
    fn from(tree: lana_app::chart_of_accounts::tree::ChartTreeControlSubAccount) -> Self {
        ChartControlSubAccount {
            name: tree.name,
            account_code: tree.encoded_path,
        }
    }
}
