use async_graphql::*;

use crate::primitives::*;
pub use lana_app::fiscal_year::{FiscalYear as DomainFiscalYear, FiscalYearsByCreatedAtCursor};

#[derive(SimpleObject, Clone)]
pub struct FiscalYear {
    id: ID,
    chart_id: UUID,
    opened_as_of: Date,
    is_open: bool,
    #[graphql(skip)]
    pub(crate) entity: Arc<DomainFiscalYear>,
}
impl From<DomainFiscalYear> for FiscalYear {
    fn from(fiscal_year: DomainFiscalYear) -> Self {
        FiscalYear {
            id: fiscal_year.id.to_global_id(),
            chart_id: UUID::from(fiscal_year.chart_id),
            opened_as_of: fiscal_year.opened_as_of.into(),
            is_open: fiscal_year.is_open(),
            entity: Arc::new(fiscal_year),
        }
    }
}

#[derive(InputObject)]
pub struct FiscalYearCloseMonthInput {
    pub fiscal_year_id: UUID,
}

crate::mutation_payload! { FiscalYearCloseMonthPayload, fiscal_year: FiscalYear }

#[derive(InputObject)]
pub struct FiscalYearInitInput {
    pub opened_as_of: Date,
}

crate::mutation_payload! { FiscalYearInitPayload, fiscal_year: FiscalYear }
