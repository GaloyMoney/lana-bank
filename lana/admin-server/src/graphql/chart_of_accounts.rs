use async_graphql::*;

use lana_app::chart_of_accounts::tree::*;

pub struct ChartOfAccounts(ChartTree);
pub struct ChartCategories(ChartTree);
pub struct ChartCategory(ChartTreeCategory);
pub struct ChartControlAccount(ChartTreeControlAccount);
pub struct ChartControlSubAccount(ChartTreeControlSubAccount);

impl From<ChartTree> for ChartOfAccounts {
    fn from(tree: ChartTree) -> Self {
        ChartOfAccounts(tree)
    }
}

#[Object]
impl ChartOfAccounts {
    async fn name(&self) -> &String {
        &self.0.name
    }

    async fn categories(&self) -> ChartCategories {
        ChartCategories(self.0.clone())
    }
}

#[Object]
impl ChartCategories {
    async fn assets(&self) -> ChartCategory {
        ChartCategory(self.0.assets.clone())
    }

    async fn liabilities(&self) -> ChartCategory {
        ChartCategory(self.0.liabilities.clone())
    }

    async fn equity(&self) -> ChartCategory {
        ChartCategory(self.0.equity.clone())
    }

    async fn revenues(&self) -> ChartCategory {
        ChartCategory(self.0.revenues.clone())
    }

    async fn expenses(&self) -> ChartCategory {
        ChartCategory(self.0.expenses.clone())
    }
}

#[Object]
impl ChartCategory {
    async fn name(&self) -> &String {
        &self.0.name
    }

    async fn account_code(&self) -> &String {
        &self.0.account_code
    }

    async fn control_accounts(&self) -> Vec<ChartControlAccount> {
        self.0
            .control_accounts
            .iter()
            .map(|a| ChartControlAccount(a.clone()))
            .collect()
    }
}

#[Object]
impl ChartControlAccount {
    async fn name(&self) -> &String {
        &self.0.name
    }

    async fn account_code(&self) -> &String {
        &self.0.account_code
    }

    async fn control_sub_accounts(&self) -> Vec<ChartControlSubAccount> {
        self.0
            .control_sub_accounts
            .iter()
            .map(|a| ChartControlSubAccount(a.clone()))
            .collect()
    }
}

#[Object]
impl ChartControlSubAccount {
    async fn name(&self) -> &String {
        &self.0.name
    }

    async fn account_code(&self) -> &String {
        &self.0.account_code
    }
}
