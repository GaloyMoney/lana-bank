use async_graphql::*;

use crate::primitives::*;

pub use lana_app::credit::ObligationAllocation as DomainObligationAllocation;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityObligationAllocation {
    id: ID,
    obligation_allocation_id: UUID,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainObligationAllocation>,
}

impl From<DomainObligationAllocation> for CreditFacilityObligationAllocation {
    fn from(allocation: DomainObligationAllocation) -> Self {
        Self {
            id: allocation.id.to_global_id(),
            obligation_allocation_id: UUID::from(allocation.id),
            amount: allocation.amount,
            created_at: allocation.created_at().into(),
            entity: Arc::new(allocation),
        }
    }
}

#[ComplexObject]
impl CreditFacilityObligationAllocation {
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
