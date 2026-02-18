use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use money::{Satoshis, UsdCents};

use crate::primitives::*;

use super::{
    PublicCollateral, PublicCreditFacility, PublicCreditFacilityProposal, PublicDisbursal,
    PublicInterestAccrualCycle, PublicPendingCreditFacility,
};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCreditEvent {
    FacilityProposalCreated {
        entity: PublicCreditFacilityProposal,
    },
    FacilityProposalConcluded {
        entity: PublicCreditFacilityProposal,
    },
    PendingCreditFacilityCollateralizationChanged {
        id: PendingCreditFacilityId,
        state: PendingCreditFacilityCollateralizationState,
        collateral: Satoshis,
        price: PriceOfOneBTC,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    PendingCreditFacilityCompleted {
        entity: PublicPendingCreditFacility,
    },
    FacilityActivated {
        entity: PublicCreditFacility,
    },
    FacilityCompleted {
        entity: PublicCreditFacility,
    },
    FacilityCollateralUpdated {
        entity: PublicCollateral,
    },
    FacilityCollateralizationChanged {
        entity: PublicCreditFacility,
    },
    DisbursalSettled {
        entity: PublicDisbursal,
    },
    AccrualPosted {
        entity: PublicInterestAccrualCycle,
    },
    PartialLiquidationInitiated {
        entity: PublicCreditFacility,
    },
    PartialLiquidationCollateralSentOut {
        liquidation_id: LiquidationId,
        credit_facility_id: CreditFacilityId,
        amount: Satoshis,
        ledger_tx_id: LedgerTxId,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    PartialLiquidationProceedsReceived {
        liquidation_id: LiquidationId,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
        payment_id: PaymentId,
        ledger_tx_id: LedgerTxId,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    PartialLiquidationCompleted {
        liquidation_id: LiquidationId,
        credit_facility_id: CreditFacilityId,
    },
}
