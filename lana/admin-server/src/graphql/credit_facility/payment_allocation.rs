use async_graphql::*;

use crate::primitives::*;

pub use lana_app::credit::PaymentAllocation as DomainPaymentAllocation;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityPaymentAllocation {
    id: ID,
    payment_allocation_id: UUID,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainPaymentAllocation>,
}

impl From<DomainPaymentAllocation> for CreditFacilityPaymentAllocation {
    fn from(payment_allocation: DomainPaymentAllocation) -> Self {
        Self {
            id: payment_allocation.id.to_global_id(),
            payment_allocation_id: UUID::from(payment_allocation.id),
            amount: payment_allocation.amount,
            created_at: payment_allocation.created_at().into(),
            entity: Arc::new(payment_allocation),
        }
    }
}

#[ComplexObject]
impl CreditFacilityPaymentAllocation {
    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<super::CreditFacility> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let cf = app
            .credit()
            .for_subject(sub)?
            .find_by_id(self.entity.credit_facility_id)
            .await?
            .expect("facility should exist for a payment");
        Ok(super::CreditFacility::from(cf))
    }
}
