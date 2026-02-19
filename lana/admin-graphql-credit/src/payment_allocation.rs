use async_graphql::*;

use crate::primitives::*;

pub use lana_app::credit::PaymentAllocation as DomainPaymentAllocation;

#[derive(SimpleObject, Clone)]
pub struct CreditFacilityPaymentAllocationBase {
    id: ID,
    payment_allocation_id: UUID,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainPaymentAllocation>,
}

impl From<DomainPaymentAllocation> for CreditFacilityPaymentAllocationBase {
    fn from(allocation: DomainPaymentAllocation) -> Self {
        Self {
            id: allocation.id.to_global_id(),
            payment_allocation_id: UUID::from(allocation.id),
            amount: allocation.amount,
            created_at: allocation.created_at().into(),
            entity: Arc::new(allocation),
        }
    }
}
