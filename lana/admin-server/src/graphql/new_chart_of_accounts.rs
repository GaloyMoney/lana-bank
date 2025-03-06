use async_graphql::*;

use crate::primitives::*;

use lana_app::new_chart_of_accounts::Chart as DomainChart;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct NewChartOfAccounts {
    id: ID,
    name: String,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainChart>,
}

impl From<DomainChart> for NewChartOfAccounts {
    fn from(chart: DomainChart) -> Self {
        NewChartOfAccounts {
            id: chart.id.to_global_id(),
            name: chart.name.to_string(),

            entity: Arc::new(chart),
        }
    }
}

#[ComplexObject]
impl NewChartOfAccounts {
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

impl From<lana_app::new_chart_of_accounts::tree::TreeNode> for ChartNode {
    fn from(node: lana_app::new_chart_of_accounts::tree::TreeNode) -> Self {
        Self {
            name: node.name.to_string(),
            account_code: node.code.to_string(),
            children: node.children.into_iter().map(ChartNode::from).collect(),
        }
    }
}
