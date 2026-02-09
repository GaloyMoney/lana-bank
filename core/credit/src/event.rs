use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_credit_terms::{InterestPeriod, collateralization::CollateralizationState};
use money::{Satoshis, UsdCents};

use crate::{
    credit_facility::CreditFacilityReceivable, ledger::FacilityProceedsFromLiquidationAccountId,
    public::{PublicCreditFacilityProposal, PublicPendingCreditFacility},
};

use super::primitives::*;

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
        id: CreditFacilityId,
        activation_tx_id: LedgerTxId,
        activated_at: DateTime<Utc>,
        amount: UsdCents,
    },
    FacilityCompleted {
        id: CreditFacilityId,
        completed_at: DateTime<Utc>,
    },
    FacilityCollateralUpdated {
        credit_facility_id: CreditFacilityId,
        pending_credit_facility_id: PendingCreditFacilityId,
        ledger_tx_id: LedgerTxId,
        new_amount: Satoshis,
        abs_diff: Satoshis,
        direction: CollateralDirection,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
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
        credit_facility_id: CreditFacilityId,
        ledger_tx_id: LedgerTxId,
        amount: UsdCents,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    AccrualPosted {
        credit_facility_id: CreditFacilityId,
        ledger_tx_id: LedgerTxId,
        amount: UsdCents,
        period: InterestPeriod,
        due_at: EffectiveDate,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
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
