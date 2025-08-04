use async_graphql::*;

use crate::primitives::*;

pub use lana_app::credit::ObligationFulfillment as DomainObligationFulfillment;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityObligationFulfillment {
    id: ID,
    obligation_fulfillment_id: UUID,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainObligationFulfillment>,
}

impl From<DomainObligationFulfillment> for CreditFacilityObligationFulfillment {
    fn from(fulfillment: DomainObligationFulfillment) -> Self {
        Self {
            id: fulfillment.id.to_global_id(),
            obligation_fulfillment_id: UUID::from(fulfillment.id),
            amount: fulfillment.amount,
            created_at: fulfillment.created_at().into(),
            entity: Arc::new(fulfillment),
        }
    }
}

#[ComplexObject]
impl CreditFacilityObligationFulfillment {
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
