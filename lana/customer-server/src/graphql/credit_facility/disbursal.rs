use async_graphql::*;

use crate::primitives::*;

pub(crate) use lana_app::credit::Disbursal as DomainDisbursal;

#[derive(SimpleObject, Clone)]
pub(crate) struct CreditFacilityDisbursal {
    id: ID,
    disbursal_id: UUID,
    amount: UsdCents,
    status: DisbursalStatus,
    created_at: Timestamp,
}

impl From<DomainDisbursal> for CreditFacilityDisbursal {
    fn from(disbursal: DomainDisbursal) -> Self {
        Self {
            id: disbursal.id.to_global_id(),
            disbursal_id: UUID::from(disbursal.id),
            amount: disbursal.amount,
            status: disbursal.status(),
            created_at: disbursal.created_at().into(),
        }
    }
}
