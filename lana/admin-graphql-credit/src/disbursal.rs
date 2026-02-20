use async_graphql::*;

use crate::{credit_facility::CreditFacilityBase, primitives::*};
pub use lana_app::{
    credit::{Disbursal as DomainDisbursal, DisbursalsCursor},
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(name = "CreditFacilityDisbursal", complex)]
pub struct CreditFacilityDisbursalBase {
    id: ID,
    disbursal_id: UUID,
    amount: UsdCents,
    status: DisbursalStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainDisbursal>,
}

impl From<DomainDisbursal> for CreditFacilityDisbursalBase {
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
impl CreditFacilityDisbursalBase {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<CreditFacilityBase> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let facilities: std::collections::HashMap<_, CreditFacilityBase> = app
            .credit()
            .facilities()
            .find_all(&[self.entity.facility_id])
            .await?;
        Ok(facilities
            .into_values()
            .next()
            .expect("disbursal must have a credit facility"))
    }

    async fn approval_process(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<admin_graphql_governance::ApprovalProcess> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let processes: std::collections::HashMap<_, admin_graphql_governance::ApprovalProcess> =
            app.governance()
                .find_all_approval_processes(&[self.entity.approval_process_id])
                .await?;
        Ok(processes
            .into_values()
            .next()
            .expect("disbursal must have an approval process"))
    }

    async fn ledger_transactions(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<admin_graphql_accounting::LedgerTransaction>> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let tx_ids = self.entity.ledger_tx_ids();
        let loaded_transactions: std::collections::HashMap<
            _,
            admin_graphql_accounting::LedgerTransaction,
        > = app
            .accounting()
            .ledger_transactions()
            .find_all(&tx_ids)
            .await?;

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
