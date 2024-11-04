use async_graphql::*;

use crate::primitives::*;
pub use lava_app::primitives::CollateralAction;

#[derive(async_graphql::Union)]
pub enum CreditFacilityHistoryEntry {
    Payment(CreditFacilityIncrementalPayment),
    Collateral(CreditFacilityCollateralUpdated),
    Origination(CreditFacilityOrigination),
    Collateralization(CreditFacilityCollateralizationUpdated),
    Disburssal(CreditFacilityDisbursalExecuted),
}

#[derive(SimpleObject)]
pub struct CreditFacilityIncrementalPayment {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityCollateralUpdated {
    pub satoshis: Satoshis,
    pub recorded_at: Timestamp,
    pub action: CollateralAction,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityOrigination {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityCollateralizationUpdated {
    pub state: CollateralizationState,
    pub collateral: Satoshis,
    pub outstanding_interest: UsdCents,
    pub outstanding_disbursement: UsdCents,
    pub recorded_at: Timestamp,
    pub price: UsdCents,
}

#[derive(SimpleObject)]
pub struct CreditFacilityDisbursalExecuted {
    pub cents: UsdCents,
    pub recorded_at: Timestamp,
    pub tx_id: UUID,
}

impl From<lava_app::credit_facility::CreditFacilityHistoryEntry> for CreditFacilityHistoryEntry {
    fn from(transaction: lava_app::credit_facility::CreditFacilityHistoryEntry) -> Self {
        match transaction {
            lava_app::credit_facility::CreditFacilityHistoryEntry::Payment(payment) => {
                CreditFacilityHistoryEntry::Payment(payment.into())
            }
            lava_app::credit_facility::CreditFacilityHistoryEntry::Collateral(collateral) => {
                CreditFacilityHistoryEntry::Collateral(collateral.into())
            }
            lava_app::credit_facility::CreditFacilityHistoryEntry::Origination(origination) => {
                CreditFacilityHistoryEntry::Origination(origination.into())
            }
            lava_app::credit_facility::CreditFacilityHistoryEntry::Collateralization(
                collateralization,
            ) => CreditFacilityHistoryEntry::Collateralization(collateralization.into()),
            lava_app::credit_facility::CreditFacilityHistoryEntry::Disbursal(disbursement) => {
                CreditFacilityHistoryEntry::Disburssal(disbursement.into())
            }
        }
    }
}

impl From<lava_app::credit_facility::IncrementalPayment> for CreditFacilityIncrementalPayment {
    fn from(payment: lava_app::credit_facility::IncrementalPayment) -> Self {
        Self {
            cents: payment.cents,
            recorded_at: payment.recorded_at.into(),
            tx_id: UUID::from(payment.tx_id),
        }
    }
}

impl From<lava_app::credit_facility::CollateralUpdated> for CreditFacilityCollateralUpdated {
    fn from(collateral: lava_app::credit_facility::CollateralUpdated) -> Self {
        Self {
            satoshis: collateral.satoshis,
            recorded_at: collateral.recorded_at.into(),
            action: collateral.action,
            tx_id: UUID::from(collateral.tx_id),
        }
    }
}

impl From<lava_app::credit_facility::CreditFacilityOrigination> for CreditFacilityOrigination {
    fn from(origination: lava_app::credit_facility::CreditFacilityOrigination) -> Self {
        Self {
            cents: origination.cents,
            recorded_at: origination.recorded_at.into(),
            tx_id: UUID::from(origination.tx_id),
        }
    }
}

impl From<lava_app::credit_facility::CollateralizationUpdated>
    for CreditFacilityCollateralizationUpdated
{
    fn from(collateralization: lava_app::credit_facility::CollateralizationUpdated) -> Self {
        Self {
            state: collateralization.state,
            collateral: collateralization.collateral,
            outstanding_interest: collateralization.outstanding_interest,
            outstanding_disbursement: collateralization.outstanding_disbursal,
            recorded_at: collateralization.recorded_at.into(),
            price: collateralization.price.into_inner(),
        }
    }
}

impl From<lava_app::credit_facility::DisbursalExecuted> for CreditFacilityDisbursalExecuted {
    fn from(disbursement: lava_app::credit_facility::DisbursalExecuted) -> Self {
        Self {
            cents: disbursement.cents,
            recorded_at: disbursement.recorded_at.into(),
            tx_id: UUID::from(disbursement.tx_id),
        }
    }
}
