use async_graphql::*;

use crate::primitives::*;

pub use lana_app::credit_facility::Payment as DomainPayment;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityPayment {
    id: ID,
    payment_id: UUID,
    interest_amount: UsdCents,
    disbursal_amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainPayment>,
}

impl From<DomainPayment> for CreditFacilityPayment {
    fn from(payment: DomainPayment) -> Self {
        Self {
            id: payment.id.to_global_id(),
            payment_id: UUID::from(payment.id),
            interest_amount: payment.amounts.interest,
            disbursal_amount: payment.amounts.disbursal,
            created_at: payment.created_at().into(),
            entity: Arc::new(payment),
        }
    }
}

#[ComplexObject]
impl CreditFacilityPayment {
    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<super::CreditFacility> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let cf = app
            .credit_facilities()
            .for_subject(sub)?
            .find_by_id(self.entity.facility_id)
            .await?
            .expect("facility should exist for a payment");
        Ok(super::CreditFacility::from(cf))
    }
}
