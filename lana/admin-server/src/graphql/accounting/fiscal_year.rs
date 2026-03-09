use async_graphql::{connection::*, *};
use es_entity::Sort;

use crate::{
    graphql::event_timeline::{self, EventTimelineCursor, EventTimelineEntry},
    primitives::*,
};
pub use lana_app::fiscal_year::{
    FiscalMonthClosure as DomainFiscalMonthClosure, FiscalYear as DomainFiscalYear,
    FiscalYearsCursor as DomainFiscalYearsCursor, FiscalYearsSortBy as DomainFiscalYearsSortBy,
};

use crate::graphql::primitives::SortDirection;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct FiscalYear {
    id: ID,
    fiscal_year_id: UUID,
    chart_id: UUID,
    is_open: bool,
    opened_as_of: Date,
    closed_as_of: Option<Date>,
    created_at: Timestamp,
    #[graphql(skip)]
    pub(crate) entity: Arc<DomainFiscalYear>,
}
impl From<DomainFiscalYear> for FiscalYear {
    fn from(fiscal_year: DomainFiscalYear) -> Self {
        FiscalYear {
            id: fiscal_year.id.to_global_id(),
            fiscal_year_id: UUID::from(fiscal_year.id),
            chart_id: UUID::from(fiscal_year.chart_id),
            is_open: fiscal_year.closed_as_of.is_none(),
            opened_as_of: fiscal_year.opened_as_of.into(),
            closed_as_of: fiscal_year.closed_as_of.map(|date| date.into()),
            created_at: fiscal_year.created_at().into(),
            entity: Arc::new(fiscal_year),
        }
    }
}

#[ComplexObject]
impl FiscalYear {
    pub async fn reference(&self) -> &str {
        &self.entity.reference
    }

    pub async fn year(&self) -> &str {
        &self.entity.year
    }

    pub async fn month_closures(&self) -> Vec<FiscalMonthClosure> {
        self.entity
            .month_closures()
            .into_iter()
            .map(FiscalMonthClosure::from)
            .collect()
    }

    pub async fn is_last_month_of_year_closed(&self) -> bool {
        self.entity.is_last_month_of_year_closed()
    }

    pub async fn next_month_to_close(&self) -> Option<Date> {
        self.entity.next_month_to_close().map(|date| date.into())
    }

    async fn event_history(
        &self,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<EventTimelineCursor, EventTimelineEntry, EmptyFields, EmptyFields>,
    > {
        use es_entity::EsEntity as _;
        event_timeline::events_to_connection(self.entity.events(), first, after)
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

#[derive(InputObject)]
pub struct FiscalYearCloseInput {
    pub fiscal_year_id: UUID,
}

crate::mutation_payload! { FiscalYearClosePayload, fiscal_year: FiscalYear }

#[derive(InputObject)]
pub struct FiscalYearOpenNextInput {
    pub fiscal_year_id: UUID,
}

crate::mutation_payload! { FiscalYearOpenNextPayload, fiscal_year: FiscalYear }

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FiscalYearsSortBy {
    #[default]
    CreatedAt,
    OpenedAsOf,
}

impl From<FiscalYearsSortBy> for DomainFiscalYearsSortBy {
    fn from(by: FiscalYearsSortBy) -> Self {
        match by {
            FiscalYearsSortBy::CreatedAt => DomainFiscalYearsSortBy::CreatedAt,
            FiscalYearsSortBy::OpenedAsOf => DomainFiscalYearsSortBy::OpenedAsOf,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct FiscalYearsSort {
    #[graphql(default)]
    pub by: FiscalYearsSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<FiscalYearsSort> for DomainFiscalYearsSortBy {
    fn from(sort: FiscalYearsSort) -> Self {
        sort.by.into()
    }
}

impl From<FiscalYearsSort> for Sort<DomainFiscalYearsSortBy> {
    fn from(sort: FiscalYearsSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}
