use async_graphql::*;

use crate::primitives::*;

pub(crate) use lana_app::credit::PaymentAllocation as DomainPaymentAllocation;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(crate) struct CreditFacilityPaymentAllocation {
    id: ID,
    payment_allocation_id: UUID,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainPaymentAllocation>,
}

impl From<DomainPaymentAllocation> for CreditFacilityPaymentAllocation {
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
impl CreditFacilityPaymentAllocation {
    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<super::CreditFacility> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let cf = app
            .credit()
            .for_subject(sub)?
            .find_by_id(self.entity.beneficiary_id)
            .await?
            .expect("facility should exist for a payment");
        Ok(super::CreditFacility::from(cf))
    }
}
