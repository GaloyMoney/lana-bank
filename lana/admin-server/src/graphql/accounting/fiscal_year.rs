use async_graphql::*;

use crate::primitives::*;
pub use lana_app::fiscal_year::{
    FiscalMonthClosure as DomainFiscalMonthClosure, FiscalYear as DomainFiscalYear,
    FiscalYearsByCreatedAtCursor,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct FiscalYear {
    id: ID,
    fiscal_year_id: UUID,
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
            fiscal_year_id: UUID::from(fiscal_year.id),
            chart_id: UUID::from(fiscal_year.chart_id),
            opened_as_of: fiscal_year.opened_as_of.into(),
            is_open: fiscal_year.is_open(),
            entity: Arc::new(fiscal_year),
        }
    }
}

#[ComplexObject]
impl FiscalYear {
    pub async fn month_closures(&self) -> Vec<FiscalMonthClosure> {
        self.entity
            .month_closures()
            .into_iter()
            .map(FiscalMonthClosure::from)
            .collect()
    }
}

#[derive(SimpleObject, Clone)]
pub struct FiscalMonthClosure {
    closed_as_of: Date,
    closed_at: Timestamp,
}

impl From<DomainFiscalMonthClosure> for FiscalMonthClosure {
    fn from(closure: DomainFiscalMonthClosure) -> Self {
        FiscalMonthClosure {
            closed_as_of: closure.closed_as_of.into(),
            closed_at: closure.closed_at.into(),
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
