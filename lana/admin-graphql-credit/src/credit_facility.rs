use async_graphql::*;

use crate::primitives::*;

pub use admin_graphql_shared::credit::CreditFacilityBase;

pub use lana_app::{
    credit::{
        CreditFacilitiesCursor, CreditFacilitiesFilters as DomainCreditFacilitiesFilters,
        CreditFacilitiesSortBy as DomainCreditFacilitiesSortBy,
        CreditFacility as DomainCreditFacility, DisbursalsFilters,
        DisbursalsSortBy as DomainDisbursalsSortBy, ListDirection, Sort,
    },
    public_id::PublicId,
};

#[derive(InputObject)]
pub struct CreditFacilityPartialPaymentRecordInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
}

#[derive(InputObject)]
pub struct CreditFacilityPartialPaymentWithDateRecordInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
    pub effective: Date,
}

#[derive(InputObject)]
pub struct CreditFacilityCompleteInput {
    pub credit_facility_id: UUID,
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreditFacilitiesSortBy {
    #[default]
    CreatedAt,
    Cvl,
}

impl From<CreditFacilitiesSortBy> for DomainCreditFacilitiesSortBy {
    fn from(by: CreditFacilitiesSortBy) -> Self {
        match by {
            CreditFacilitiesSortBy::CreatedAt => DomainCreditFacilitiesSortBy::CreatedAt,
            CreditFacilitiesSortBy::Cvl => DomainCreditFacilitiesSortBy::CollateralizationRatio,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct CreditFacilitiesSort {
    #[graphql(default)]
    pub by: CreditFacilitiesSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<CreditFacilitiesSort> for Sort<DomainCreditFacilitiesSortBy> {
    fn from(sort: CreditFacilitiesSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<CreditFacilitiesSort> for DomainCreditFacilitiesSortBy {
    fn from(sort: CreditFacilitiesSort) -> Self {
        sort.by.into()
    }
}

#[derive(InputObject)]
pub struct CreditFacilitiesFilter {
    pub status: Option<CreditFacilityStatus>,
    pub collateralization_state: Option<CollateralizationState>,
}
