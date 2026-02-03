use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_money::{Satoshis, UsdCents};

use crate::{
    CollateralizationState, CreditFacilityReceivable, FacilityProceedsFromLiquidationAccountId,
    InterestPeriod, TermValues,
};

use super::primitives::*;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCreditEvent {
    FacilityProposalCreated {
        id: CreditFacilityProposalId,
        terms: TermValues,
        amount: UsdCents,
        created_at: DateTime<Utc>,
    },
    FacilityProposalConcluded {
        id: CreditFacilityProposalId,
        status: CreditFacilityProposalStatus,
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
        id: PendingCreditFacilityId,
        status: PendingCreditFacilityStatus,
        recorded_at: DateTime<Utc>,
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
    FacilityPaymentAllocated {
        credit_facility_id: CreditFacilityId,
        obligation_id: ObligationId,
        obligation_type: ObligationType,
        allocation_id: PaymentAllocationId,
        amount: UsdCents,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
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
    ObligationCreated {
        id: ObligationId,
        obligation_type: ObligationType,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,

        due_at: EffectiveDate,
        overdue_at: Option<EffectiveDate>,
        defaulted_at: Option<EffectiveDate>,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    ObligationDue {
        id: ObligationId,
        credit_facility_id: CreditFacilityId,
        obligation_type: ObligationType,
        amount: UsdCents,
    },
    ObligationOverdue {
        id: ObligationId,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
    },
    ObligationDefaulted {
        id: ObligationId,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
    },
    ObligationCompleted {
        id: ObligationId,
        credit_facility_id: CreditFacilityId,
    },
    PartialLiquidationInitiated {
        liquidation_id: LiquidationId,
        credit_facility_id: CreditFacilityId,
        collateral_id: CollateralId,
        customer_id: CustomerId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
        collateral_account_id: CalaAccountId,
        collateral_in_liquidation_account_id: CalaAccountId,
        liquidated_collateral_account_id: CalaAccountId,
        proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
        payment_holding_account_id: CalaAccountId,
        uncovered_outstanding_account_id: CalaAccountId,
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
