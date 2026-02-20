use async_graphql::*;

use crate::primitives::*;

pub use admin_graphql_shared::credit::CreditFacilityDisbursalBase;
pub use lana_app::{
    credit::{Disbursal as DomainDisbursal, DisbursalsCursor},
    public_id::PublicId,
};

#[derive(InputObject)]
pub struct CreditFacilityDisbursalInitiateInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
}
