use async_graphql::*;

pub use lana_app::primitives::CollateralAction;

use super::{CreditFacilityDisbursal, CreditFacilityPaymentAllocation, LanaDataLoader};
use crate::primitives::*;

#[derive(async_graphql::Union)]
pub enum CreditFacilityHistoryEntry {
    Payment(CreditFacilityIncrementalPayment),
    Collateral(CreditFacilityCollateralUpdated),
    Approved(CreditFacilityApproved),
    Collateralization(CreditFacilityCollateralizationUpdated),
    Disbursal(CreditFacilityDisbursalExecuted),
    Interest(CreditFacilityInterestAccrued),
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CreditFacilityIncrementalPayment {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub effective: Date,
    pub tx_id: UUID,
    #[graphql(skip)]
    payment_allocation_id: PaymentAllocationId,
}

#[ComplexObject]
impl CreditFacilityIncrementalPayment {
    async fn payment(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<CreditFacilityPaymentAllocation> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();

        let payment_allocation = loader
            .load_one(self.payment_allocation_id)
            .await?
            .expect("payment allocation should exist");

        Ok(payment_allocation)
    }
}

#[derive(SimpleObject)]
pub struct CreditFacilityCollateralUpdated {
    pub satoshis: Satoshis,
    pub recorded_at: Timestamp,
    pub effective: Date,
    pub action: CollateralAction,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityApproved {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub effective: Date,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityCollateralizationUpdated {
    pub state: CollateralizationState,
    pub collateral: Satoshis,
    pub outstanding_interest: UsdCents,
    pub outstanding_disbursal: UsdCents,
    pub recorded_at: Timestamp,
    pub effective: Date,
    pub price: UsdCents,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CreditFacilityDisbursalExecuted {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub effective: Date,
    pub tx_id: UUID,
}

#[ComplexObject]
impl CreditFacilityDisbursalExecuted {
    async fn disbursal(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacilityDisbursal> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let disbursal = app
            .credit()
            .disbursals()
            .find_by_concluded_tx_id(sub, self.tx_id)
            .await?;

        Ok(CreditFacilityDisbursal::from(disbursal))
    }
}

#[derive(SimpleObject)]
pub struct CreditFacilityInterestAccrued {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub effective: Date,
    pub tx_id: UUID,
    pub days: u32,
}

impl From<lana_app::credit::CreditFacilityHistoryEntry> for CreditFacilityHistoryEntry {
    fn from(transaction: lana_app::credit::CreditFacilityHistoryEntry) -> Self {
        match transaction {
            lana_app::credit::CreditFacilityHistoryEntry::Payment(payment) => {
                CreditFacilityHistoryEntry::Payment(payment.into())
            }
            lana_app::credit::CreditFacilityHistoryEntry::Collateral(collateral) => {
                CreditFacilityHistoryEntry::Collateral(collateral.into())
            }
            lana_app::credit::CreditFacilityHistoryEntry::Approved(approved) => {
                CreditFacilityHistoryEntry::Approved(approved.into())
            }
            lana_app::credit::CreditFacilityHistoryEntry::Collateralization(collateralization) => {
                CreditFacilityHistoryEntry::Collateralization(collateralization.into())
            }
            lana_app::credit::CreditFacilityHistoryEntry::Disbursal(disbursal) => {
                CreditFacilityHistoryEntry::Disbursal(disbursal.into())
            }
            lana_app::credit::CreditFacilityHistoryEntry::Interest(interest) => {
                CreditFacilityHistoryEntry::Interest(interest.into())
            }
        }
    }
}

impl From<lana_app::credit::IncrementalPayment> for CreditFacilityIncrementalPayment {
    fn from(payment: lana_app::credit::IncrementalPayment) -> Self {
        Self {
            cents: payment.cents,
            recorded_at: payment.recorded_at.into(),
            effective: payment.effective.into(),
            tx_id: UUID::from(payment.payment_id),
            payment_allocation_id: payment.payment_id,
        }
    }
}

impl From<lana_app::credit::CollateralUpdated> for CreditFacilityCollateralUpdated {
    fn from(collateral: lana_app::credit::CollateralUpdated) -> Self {
        Self {
            satoshis: collateral.satoshis,
            recorded_at: collateral.recorded_at.into(),
            effective: collateral.effective.into(),
            action: collateral.action,
            tx_id: UUID::from(collateral.tx_id),
        }
    }
}

impl From<lana_app::credit::CreditFacilityApproved> for CreditFacilityApproved {
    fn from(origination: lana_app::credit::CreditFacilityApproved) -> Self {
        Self {
            cents: origination.cents,
            recorded_at: origination.recorded_at.into(),
            effective: origination.effective.into(),
            tx_id: UUID::from(origination.tx_id),
        }
    }
}

impl From<lana_app::credit::CollateralizationUpdated> for CreditFacilityCollateralizationUpdated {
    fn from(collateralization: lana_app::credit::CollateralizationUpdated) -> Self {
        Self {
            state: collateralization.state,
            collateral: collateralization.collateral,
            outstanding_interest: collateralization.outstanding_interest,
            outstanding_disbursal: collateralization.outstanding_disbursal,
            recorded_at: collateralization.recorded_at.into(),
            effective: collateralization.effective.into(),
            price: collateralization.price.into_inner(),
        }
    }
}

impl From<lana_app::credit::DisbursalExecuted> for CreditFacilityDisbursalExecuted {
    fn from(disbursal: lana_app::credit::DisbursalExecuted) -> Self {
        Self {
            cents: disbursal.cents,
            recorded_at: disbursal.recorded_at.into(),
            effective: disbursal.effective.into(),
            tx_id: UUID::from(disbursal.tx_id),
        }
    }
}

impl From<lana_app::credit::InterestAccrualsPosted> for CreditFacilityInterestAccrued {
    fn from(interest: lana_app::credit::InterestAccrualsPosted) -> Self {
        Self {
            cents: interest.cents,
            recorded_at: interest.recorded_at.into(),
            effective: interest.effective.into(),
            tx_id: UUID::from(interest.tx_id),
            days: interest.days,
        }
    }
}
