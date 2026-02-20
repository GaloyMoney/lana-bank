use async_graphql::*;

use crate::{graphql::loader::LanaDataLoader, primitives::*};

// Re-export base types and value types from the credit crate
pub use admin_graphql_credit::{
    CollateralBase, CreditFacilityBase, CreditFacilityCollateralizationUpdated,
    CreditFacilityDisbursalBase, CreditFacilityProposalBase, LiquidationBase,
    PendingCreditFacilityBase, PendingCreditFacilityCollateralizationUpdated,
};

// ===== Type aliases =====

pub type CreditFacility = CreditFacilityBase;
pub type CreditFacilityProposal = CreditFacilityProposalBase;
pub type PendingCreditFacility = PendingCreditFacilityBase;
pub type CreditFacilityDisbursal = CreditFacilityDisbursalBase;
pub type Collateral = CollateralBase;
pub type Liquidation = LiquidationBase;

// ===== Subscription payload types =====

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CreditFacilityProposalConcludedPayload {
    pub status: CreditFacilityProposalStatus,
    #[graphql(skip)]
    pub credit_facility_proposal_id: CreditFacilityProposalId,
}

#[ComplexObject]
impl CreditFacilityProposalConcludedPayload {
    async fn credit_facility_proposal(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<CreditFacilityProposal> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let proposal = loader
            .load_one(self.credit_facility_proposal_id)
            .await?
            .expect("credit facility proposal not found");
        Ok(proposal)
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PendingCreditFacilityCollateralizationPayload {
    #[graphql(flatten)]
    pub update: PendingCreditFacilityCollateralizationUpdated,
    #[graphql(skip)]
    pub pending_credit_facility_id: PendingCreditFacilityId,
}

#[ComplexObject]
impl PendingCreditFacilityCollateralizationPayload {
    async fn pending_credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<PendingCreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.pending_credit_facility_id)
            .await?
            .expect("pending credit facility not found");
        Ok(facility)
    }
}

#[derive(SimpleObject)]
pub struct PendingCreditFacilityCompleted {
    pub status: PendingCreditFacilityStatus,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PendingCreditFacilityCompletedPayload {
    #[graphql(flatten)]
    pub update: PendingCreditFacilityCompleted,
    #[graphql(skip)]
    pub pending_credit_facility_id: PendingCreditFacilityId,
}

#[ComplexObject]
impl PendingCreditFacilityCompletedPayload {
    async fn pending_credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<PendingCreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.pending_credit_facility_id)
            .await?
            .expect("pending credit facility not found");
        Ok(facility)
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CreditFacilityCollateralizationPayload {
    #[graphql(flatten)]
    pub update: CreditFacilityCollateralizationUpdated,
    #[graphql(skip)]
    pub credit_facility_id: CreditFacilityId,
}

#[ComplexObject]
impl CreditFacilityCollateralizationPayload {
    async fn credit_facility(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.credit_facility_id)
            .await?
            .expect("credit facility not found");
        Ok(facility)
    }
}
