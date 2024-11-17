use async_graphql::*;

use super::CreditFacility;
use crate::{
    graphql::{approval_process::*, loader::LavaDataLoader},
    primitives::*,
};
pub use lava_app::credit_facility::{
    Disbursal as DomainDisbursal, DisbursalsByCreatedAtCursor, DisbursalsCursor,
};

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
    async fn credit_facility(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacility> {
        let loader = ctx.data_unchecked::<LavaDataLoader>();
        let facility = loader
            .load_one(self.entity.facility_id)
            .await?
            .expect("committee not found");
        Ok(facility)
    }

    async fn status(&self, ctx: &Context<'_>) -> async_graphql::Result<DisbursalStatus> {
        let (app, _) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit_facilities()
            .ensure_up_to_date_disbursal_status(&self.entity)
            .await?
            .map(|d| d.status())
            .unwrap_or_else(|| self.entity.status()))
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LavaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }
}

#[derive(InputObject)]
pub struct CreditFacilityDisbursalInitiateInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
}
crate::mutation_payload! { CreditFacilityDisbursalInitiatePayload, disbursal: CreditFacilityDisbursal }
