use async_graphql::*;

use crate::{
    primitives::UsdCents,
    server::shared_graphql::{convert::ToGlobalId, primitives::UUID},
};

#[derive(InputObject)]
pub struct CreditFacilityCreateInput {
    pub customer_id: UUID,
    pub facility: UsdCents,
    pub terms: CreditFacilityTermsInput,
}

#[derive(InputObject)]
pub struct CreditFacilityTermsInput {
    pub duration: CreditFacilityDurationInput,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum CreditFacilityPeriod {
    Months,
}

#[derive(InputObject)]
pub struct CreditFacilityDurationInput {
    pub period: CreditFacilityPeriod,
    pub units: u32,
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

#[derive(InputObject)]
pub struct CreditFacilityApproveInput {
    pub credit_facility_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityApprovePayload {
    credit_facility: CreditFacility,
}

impl From<crate::credit_facility::CreditFacility> for CreditFacilityApprovePayload {
    fn from(credit_facility: crate::credit_facility::CreditFacility) -> Self {
        Self {
            credit_facility: credit_facility.into(),
        }
    }
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

impl From<CreditFacilityDurationInput> for crate::credit_facility::Duration {
    fn from(duration: CreditFacilityDurationInput) -> Self {
        match duration.period {
            CreditFacilityPeriod::Months => {
                crate::credit_facility::Duration::Months(duration.units)
            }
        }
    }
}
