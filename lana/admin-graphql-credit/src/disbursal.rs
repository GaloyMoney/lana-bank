use async_graphql::*;

use crate::primitives::*;
pub use lana_app::{
    credit::{Disbursal as DomainDisbursal, DisbursalsCursor},
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityDisbursalBase {
    id: ID,
    disbursal_id: UUID,
    amount: UsdCents,
    status: DisbursalStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainDisbursal>,
}

impl From<DomainDisbursal> for CreditFacilityDisbursalBase {
    fn from(disbursal: DomainDisbursal) -> Self {
        Self {
            id: disbursal.id.to_global_id(),
            disbursal_id: UUID::from(disbursal.id),
            amount: disbursal.amount,
            status: disbursal.status(),
            created_at: disbursal.created_at().into(),
            entity: Arc::new(disbursal),
        }
    }
}

#[ComplexObject]
impl CreditFacilityDisbursalBase {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }
}

#[derive(InputObject)]
pub struct CreditFacilityDisbursalInitiateInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
}
