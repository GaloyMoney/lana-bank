use async_graphql::*;

use crate::{
    primitives::UsdCents,
    server::shared_graphql::{convert::ToGlobalId, primitives::UUID},
};

#[derive(InputObject)]
pub struct CreditFacilityCreateInput {
    customer_id: UUID,
    amount: UsdCents,
}

#[derive(SimpleObject, Clone)]
pub struct CreditFacility {
    id: ID,
    credit_facility_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityCreatePayload {
    credit_facility: CreditFacility,
}

impl ToGlobalId for crate::primitives::CreditFacilityId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("credit-facility:{}", self))
    }
}

impl From<crate::credit_facility::CreditFacility> for CreditFacility {
    fn from(credit_facility: crate::credit_facility::CreditFacility) -> Self {
        Self {
            id: credit_facility.id.to_global_id(),
            credit_facility_id: UUID::from(credit_facility.id),
        }
    }
}

impl From<crate::credit_facility::CreditFacility> for CreditFacilityCreatePayload {
    fn from(credit_facility: crate::credit_facility::CreditFacility) -> Self {
        Self {
            credit_facility: CreditFacility::from(credit_facility),
        }
    }
}
