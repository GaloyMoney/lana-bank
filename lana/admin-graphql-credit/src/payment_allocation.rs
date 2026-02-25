use async_graphql::*;

use crate::primitives::*;

pub use lana_app::credit::PaymentAllocation as DomainPaymentAllocation;

#[derive(SimpleObject, Clone)]
#[graphql(name = "CreditFacilityPaymentAllocation", complex)]
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

#[ComplexObject]
impl CreditFacilityPaymentAllocationBase {
    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<crate::credit_facility::CreditFacilityBase> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let cf = app
            .credit()
            .for_subject(sub)?
            .find_by_id(self.entity.beneficiary_id)
            .await?
            .expect("facility should exist for a payment");
        Ok(crate::credit_facility::CreditFacilityBase::from(cf))
    }
}
