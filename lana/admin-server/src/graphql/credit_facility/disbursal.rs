use async_graphql::*;

use super::CreditFacility;
use crate::{
    graphql::{accounting::LedgerTransaction, approval_process::*, loader::LanaDataLoader},
    primitives::*,
};
pub(crate) use lana_app::{
    credit::{Disbursal as DomainDisbursal, DisbursalsCursor},
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityDisbursal {
    id: ID,
    disbursal_id: UUID,
    amount: UsdCents,
    status: DisbursalStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainDisbursal>,
}

impl From<DomainDisbursal> for CreditFacilityDisbursal {
    fn from(disbursal: DomainDisbursal) -> Self {
        Self {
            id: disbursal.id.to_global_id(),
            disbursal_id: UUID::from(disbursal.id),
            amount: disbursal.amount,
            status: disbursal.status(),
            created_at: disbursal.created_at().into(),
            entity: Arc::new(disbursal),
        }
    }
}

#[ComplexObject]
impl CreditFacilityDisbursal {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn credit_facility(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.entity.facility_id)
            .await?
            .expect("committee not found");
        Ok(facility)
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }

    async fn ledger_transactions(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<LedgerTransaction>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let tx_ids = self.entity.ledger_tx_ids();
        let loaded_transactions = loader.load_many(tx_ids.iter().copied()).await?;

        Ok(tx_ids
            .iter()
            .filter_map(|id| loaded_transactions.get(id).cloned())
            .collect())
    }
}

#[derive(InputObject)]
pub struct CreditFacilityDisbursalInitiateInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
}
crate::mutation_payload! { CreditFacilityDisbursalInitiatePayload, disbursal: CreditFacilityDisbursal }
