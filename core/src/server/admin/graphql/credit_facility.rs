use async_graphql::*;

use crate::{primitives::UsdCents, server::shared_graphql::primitives::UUID};

#[derive(InputObject)]
pub struct CreditFacilityCreateInput {
    customer_id: UUID,
    amount: UsdCents,
}

#[derive(SimpleObject, Clone)]
pub struct CreditFacility {
    credit_facility_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityCreatePayload {
    credit_facility: CreditFacility,
}

impl From<crate::credit_facility::CreditFacility> for CreditFacility {
    fn from(credit_facility: crate::credit_facility::CreditFacility) -> Self {
        Self {
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
