use chrono::{DateTime, Utc};

use crate::primitives::*;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CreditFacilityApproved {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
    pub tx_id: LedgerTxId,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct IncrementalPayment {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
    pub payment_id: PaymentAllocationId,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CollateralUpdated {
    pub satoshis: Satoshis,
    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
    pub action: CollateralAction,
    pub tx_id: LedgerTxId,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CollateralizationUpdated {
    pub state: CollateralizationState,
    pub collateral: Satoshis,
    pub outstanding_interest: UsdCents,
    pub outstanding_disbursal: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
    pub price: PriceOfOneBTC,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DisbursalExecuted {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
    pub tx_id: LedgerTxId,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct InterestAccrualsPosted {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
    pub days: u32,
    pub tx_id: LedgerTxId,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ObligationMovedToLiquidation {
    pub cents: UsdCents,
    pub recorded_at: DateTime<Utc>,
    pub effective: chrono::NaiveDate,
    pub tx_id: LedgerTxId,
}

/// Represents an entry in Credit Facility history as it is stored in a database.
/// The entries contain no running sums; if needed, they have to be calculated
/// during replaying.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum CreditFacilityHistoryEntry {
    Approved(CreditFacilityApproved),
    Collateral(CollateralUpdated),
    Collateralization(CollateralizationUpdated),
    Payment(IncrementalPayment),
    Disbursal(DisbursalExecuted),
    Interest(InterestAccrualsPosted),
    ReservedForLiquidation(ObligationMovedToLiquidation),
}
