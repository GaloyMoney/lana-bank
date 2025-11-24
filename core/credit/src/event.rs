use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_money::{Satoshis, UsdCents};

use crate::{CollateralizationState, CreditFacilityReceivable, TermValues, terms::InterestPeriod};

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
    FacilityProposalApproved {
        id: CreditFacilityProposalId,
    },
    PendingCreditFacilityCollateralizationChanged {
        id: PendingCreditFacilityId,
        state: PendingCreditFacilityCollateralizationState,
        collateral: Satoshis,
        price: PriceOfOneBTC,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
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
    FacilityRepaymentRecorded {
        credit_facility_id: CreditFacilityId,
        obligation_id: ObligationId,
        obligation_type: ObligationType,
        payment_id: PaymentAllocationId,
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
        action: CollateralAction,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    FacilityCollateralizationChanged {
        id: CreditFacilityId,
        state: CollateralizationState,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
        collateral: Satoshis,
        outstanding: CreditFacilityReceivable,
        cvl: CVLPct,
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
    LiquidationProcessStarted {
        id: LiquidationProcessId,
        credit_facility_id: CreditFacilityId,
        ledger_tx_id: LedgerTxId,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    LiquidationProcessConcluded {
        id: LiquidationProcessId,
        obligation_id: ObligationId,
        credit_facility_id: CreditFacilityId,
    },
}
