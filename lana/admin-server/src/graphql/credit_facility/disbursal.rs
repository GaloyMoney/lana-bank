use async_graphql::*;

use super::CreditFacility;
use crate::{
    graphql::{accounting::LedgerTransaction, approval_process::*, loader::LanaDataLoader},
    primitives::*,
};
use es_entity::Sort;

use super::SortDirection;

pub use lana_app::{
    credit::{
        Disbursal as DomainDisbursal, DisbursalsCursor, DisbursalsSortBy as DomainDisbursalsSortBy,
    },
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityDisbursal {
    id: ID,
    credit_facility_disbursal_id: UUID,
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
            credit_facility_disbursal_id: UUID::from(disbursal.id),
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

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct DisbursalApprovalConcludedPayload {
    pub status: DisbursalStatus,
    #[graphql(skip)]
    pub disbursal_id: DisbursalId,
}

#[ComplexObject]
impl DisbursalApprovalConcludedPayload {
    async fn disbursal(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacilityDisbursal> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let disbursal = loader
            .load_one(self.disbursal_id)
            .await?
            .expect("disbursal not found");
        Ok(disbursal)
    }
}

#[derive(InputObject)]
pub struct CreditFacilityDisbursalInitiateInput {
    pub credit_facility_id: UUID,
    pub amount: UsdCents,
}
crate::mutation_payload! { CreditFacilityDisbursalInitiatePayload, disbursal: CreditFacilityDisbursal }

#[derive(InputObject)]
pub struct DisbursalsFilter {
    pub status: Option<DisbursalStatus>,
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisbursalsSortBy {
    #[default]
    CreatedAt,
    Amount,
}

impl From<DisbursalsSortBy> for DomainDisbursalsSortBy {
    fn from(by: DisbursalsSortBy) -> Self {
        match by {
            DisbursalsSortBy::CreatedAt => DomainDisbursalsSortBy::CreatedAt,
            DisbursalsSortBy::Amount => DomainDisbursalsSortBy::Amount,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct DisbursalsSort {
    #[graphql(default)]
    pub by: DisbursalsSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<DisbursalsSort> for Sort<DomainDisbursalsSortBy> {
    fn from(sort: DisbursalsSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<DisbursalsSort> for DomainDisbursalsSortBy {
    fn from(sort: DisbursalsSort) -> Self {
        sort.by.into()
    }
}
