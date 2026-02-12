use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_credit_terms::collateralization::CollateralizationState;
use money::{Satoshis, UsdCents};

use crate::{
    credit_facility::CreditFacilityReceivable, ledger::FacilityProceedsFromLiquidationAccountId,
    primitives::*,
};

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
        entity: PublicPendingCreditFacility,
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
    // NOTE: `entity.adjustment` comes from collateral's latest update.
    // Current flows persist at most one manual/custodian collateral update per operation.
    FacilityCollateralUpdated {
        entity: PublicCollateral,
    },
    FacilityCollateralizationChanged {
        id: CreditFacilityId,
        customer_id: CustomerId,
        state: CollateralizationState,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
        collateral: Satoshis,
        outstanding: CreditFacilityReceivable,
        price: PriceOfOneBTC,
    },
    DisbursalSettled {
        entity: PublicDisbursal,
    },
    AccrualPosted {
        entity: PublicInterestAccrualCycle,
    },
    PartialLiquidationInitiated {
        liquidation_id: LiquidationId,
        credit_facility_id: CreditFacilityId,
        collateral_id: CollateralId,
        customer_id: CustomerId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
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
        facility_payment_holding_account_id: CalaAccountId,
        facility_proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
        facility_uncovered_outstanding_account_id: CalaAccountId,
        ledger_tx_id: LedgerTxId,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    PartialLiquidationCompleted {
        liquidation_id: LiquidationId,
        credit_facility_id: CreditFacilityId,
    },
}
