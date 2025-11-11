use async_graphql::*;

use crate::primitives::*;
use lana_app::fiscal_year::FiscalYear as DomainFiscalYear;

#[derive(SimpleObject, Clone)]
pub struct FiscalYear {
    id: ID,
    chart_id: UUID,
    first_period_opened_as_of: Date,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainFiscalYear>,
}
impl From<DomainFiscalYear> for FiscalYear {
    fn from(fiscal_year: DomainFiscalYear) -> Self {
        FiscalYear {
            id: fiscal_year.id.to_global_id(),
            chart_id: UUID::from(fiscal_year.chart_id),
            first_period_opened_as_of: fiscal_year.first_period_opened_as_of.into(),
            entity: Arc::new(fiscal_year),
        }
    }
}

#[derive(InputObject)]
pub struct FiscalYearCloseMonthInput {
    pub chart_id: UUID,
}

crate::mutation_payload! { FiscalYearCloseMonthPayload, fiscal_year: FiscalYear }

#[derive(InputObject)]
pub struct FiscalYearOpenFirstInput {
    pub chart_id: UUID,
    pub opened_as_of: Date,
}

crate::mutation_payload! { FiscalYearOpenFirstPayload, fiscal_year: FiscalYear }
