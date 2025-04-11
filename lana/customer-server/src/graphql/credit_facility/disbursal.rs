use async_graphql::*;

use crate::primitives::*;

pub use lana_app::credit::Disbursal as DomainDisbursal;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityDisbursal {
    id: ID,
    disbursal_id: UUID,
    index: DisbursalIdx,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainDisbursal>,
}

impl From<DomainDisbursal> for CreditFacilityDisbursal {
    fn from(disbursal: DomainDisbursal) -> Self {
        Self {
            id: disbursal.id.to_global_id(),
            disbursal_id: UUID::from(disbursal.id),
            index: disbursal.idx,
            amount: disbursal.amount,
            created_at: disbursal.created_at().into(),
            entity: Arc::new(disbursal),
        }
    }
}

#[ComplexObject]
impl CreditFacilityDisbursal {
    async fn status(&self, ctx: &Context<'_>) -> async_graphql::Result<DisbursalStatus> {
        let (app, _) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .ensure_up_to_date_disbursal_status(&self.entity)
            .await?
            .map(|d| d.status())
            .unwrap_or_else(|| self.entity.status()))
    }
}
