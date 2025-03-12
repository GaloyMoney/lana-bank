use async_graphql::*;

use crate::primitives::*;

use lana_app::chart_of_accounts::Chart as DomainChart;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct ChartOfAccounts {
    id: ID,
    chart_id: UUID,
    name: String,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainChart>,
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
}

#[derive(SimpleObject)]
pub struct ChartNode {
    name: String,
    account_code: String,
    children: Vec<ChartNode>,
}

impl From<lana_app::chart_of_accounts::tree::TreeNode> for ChartNode {
    fn from(node: lana_app::chart_of_accounts::tree::TreeNode) -> Self {
        Self {
            name: node.name.to_string(),
            account_code: node.code.to_string(),
            children: node.children.into_iter().map(ChartNode::from).collect(),
        }
    }
}

#[derive(InputObject)]
pub struct ChartOfAccountsCsvImportInput {
    pub chart_id: UUID,
    pub file: Upload,
}

#[derive(SimpleObject)]
pub struct ChartOfAccountsCsvImportPayload {
    pub success: bool,
}
