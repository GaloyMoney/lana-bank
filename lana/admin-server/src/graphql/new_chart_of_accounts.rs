use async_graphql::*;

use crate::primitives::*;

use lana_app::chart_of_accounts::CoreChart as DomainChart;

#[derive(SimpleObject, Clone)]
pub struct CoreChartOfAccounts {
    id: ID,
    name: String,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainChart>,
}

impl From<DomainChart> for CoreChartOfAccounts {
    fn from(chart: DomainChart) -> Self {
        CoreChartOfAccounts {
            id: chart.id.to_global_id(),
            name: chart.name.to_string(),

            _entity: Arc::new(chart),
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
