use async_graphql::*;

use crate::primitives::*;

pub use lana_app::credit::ObligationInstallment as DomainObligationInstallment;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityObligationInstallment {
    id: ID,
    obligation_installment_id: UUID,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainObligationInstallment>,
}

impl From<DomainObligationInstallment> for CreditFacilityObligationInstallment {
    fn from(installment: DomainObligationInstallment) -> Self {
        Self {
            id: installment.id.to_global_id(),
            obligation_installment_id: UUID::from(installment.id),
            amount: installment.amount,
            created_at: installment.created_at().into(),
            entity: Arc::new(installment),
        }
    }
}

#[ComplexObject]
impl CreditFacilityObligationInstallment {
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
