use async_graphql::*;

use crate::{graphql::accounting::AccountCode, primitives::*};

use lana_app::accounting::Chart as DomainChart;
use lana_app::primitives::{
    AccountInfo as DomainAccountInfo, AccountingBaseConfig, DebitOrCredit,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct ChartOfAccounts {
    id: ID,
    chart_id: UUID,
    name: String,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainChart>,
}

impl From<DomainChart> for ChartOfAccounts {
    fn from(chart: DomainChart) -> Self {
        ChartOfAccounts {
            id: chart.id.to_global_id(),
            chart_id: UUID::from(chart.id),
            name: chart.name.to_string(),
            entity: Arc::new(chart),
        }
    }
}

#[ComplexObject]
impl ChartOfAccounts {
    async fn children(&self) -> Vec<ChartNode> {
        self.entity
            .chart()
            .children
            .into_iter()
            .map(ChartNode::from)
            .collect()
    }

    async fn accounting_base_config(&self) -> Option<AccountingBaseConfigOutput> {
        self.entity
            .accounting_base_config()
            .map(AccountingBaseConfigOutput::from)
    }
}

#[derive(SimpleObject)]
pub struct AccountingBaseConfigOutput {
    pub assets_code: String,
    pub liabilities_code: String,
    pub equity_code: String,
    pub equity_retained_earnings_gain_code: String,
    pub equity_retained_earnings_loss_code: String,
    pub revenue_code: String,
    pub cost_of_revenue_code: String,
    pub expenses_code: String,
}

impl From<AccountingBaseConfig> for AccountingBaseConfigOutput {
    fn from(config: AccountingBaseConfig) -> Self {
        Self {
            assets_code: config.assets_code.to_string(),
            liabilities_code: config.liabilities_code.to_string(),
            equity_code: config.equity_code.to_string(),
            equity_retained_earnings_gain_code: config
                .equity_retained_earnings_gain_code
                .to_string(),
            equity_retained_earnings_loss_code: config
                .equity_retained_earnings_loss_code
                .to_string(),
            revenue_code: config.revenue_code.to_string(),
            cost_of_revenue_code: config.cost_of_revenue_code.to_string(),
            expenses_code: config.expenses_code.to_string(),
        }
    }
}

#[derive(SimpleObject)]
pub struct ChartNode {
    name: String,
    account_code: AccountCode,
    children: Vec<ChartNode>,
}

impl From<lana_app::accounting::tree::TreeNode> for ChartNode {
    fn from(node: lana_app::accounting::tree::TreeNode) -> Self {
        Self {
            name: node.name.to_string(),
            account_code: AccountCode::from(&node.code),
            children: node.children.into_iter().map(ChartNode::from).collect(),
        }
    }
}

#[derive(InputObject)]
pub struct ChartOfAccountsAddRootNodeInput {
    pub code: AccountCode,
    pub name: String,
    pub normal_balance_type: DebitOrCredit,
}
crate::mutation_payload! { ChartOfAccountsAddRootNodePayload, chart_of_accounts: ChartOfAccounts }

#[derive(InputObject)]
pub struct ChartOfAccountsAddChildNodeInput {
    pub parent: AccountCode,
    pub code: AccountCode,
    pub name: String,
}
crate::mutation_payload! { ChartOfAccountsAddChildNodePayload, chart_of_accounts: ChartOfAccounts }

impl TryFrom<ChartOfAccountsAddRootNodeInput> for AccountSpec {
    type Error = Box<dyn std::error::Error + Sync + Send>;

    fn try_from(input: ChartOfAccountsAddRootNodeInput) -> Result<Self, Self::Error> {
        let ChartOfAccountsAddRootNodeInput {
            code,
            name,
            normal_balance_type,
            ..
        } = input;

        Ok(Self::try_new(
            None,
            code.try_into()?,
            name.parse()?,
            normal_balance_type,
        )?)
    }
}

#[derive(InputObject)]
pub struct AccountingBaseConfigInput {
    pub assets_code: String,
    pub liabilities_code: String,
    pub equity_code: String,
    pub equity_retained_earnings_gain_code: String,
    pub equity_retained_earnings_loss_code: String,
    pub revenue_code: String,
    pub cost_of_revenue_code: String,
    pub expenses_code: String,
}

impl TryFrom<AccountingBaseConfigInput> for AccountingBaseConfig {
    type Error = Box<dyn std::error::Error + Sync + Send>;

    fn try_from(input: AccountingBaseConfigInput) -> Result<Self, Self::Error> {
        Ok(AccountingBaseConfig::try_new(
            input.assets_code.parse()?,
            input.liabilities_code.parse()?,
            input.equity_code.parse()?,
            input.equity_retained_earnings_gain_code.parse()?,
            input.equity_retained_earnings_loss_code.parse()?,
            input.revenue_code.parse()?,
            input.cost_of_revenue_code.parse()?,
            input.expenses_code.parse()?,
        )?)
    }
}

#[derive(InputObject)]
pub struct ChartOfAccountsCsvImportWithBaseConfigInput {
    pub file: Upload,
    pub base_config: AccountingBaseConfigInput,
}

crate::mutation_payload! { ChartOfAccountsCsvImportWithBaseConfigPayload, chart_of_accounts: ChartOfAccounts }

#[derive(InputObject)]
pub struct ChartOfAccountsCsvImportInput {
    pub file: Upload,
}

crate::mutation_payload! { ChartOfAccountsCsvImportPayload, chart_of_accounts: ChartOfAccounts }

#[derive(SimpleObject)]
pub struct AccountInfo {
    account_set_id: UUID,
    code: String,
    name: String,
}

impl From<DomainAccountInfo> for AccountInfo {
    fn from(info: DomainAccountInfo) -> Self {
        Self {
            account_set_id: UUID::from(info.account_set_id),
            code: info.code.to_string(),
            name: info.name.to_string(),
        }
    }
}
