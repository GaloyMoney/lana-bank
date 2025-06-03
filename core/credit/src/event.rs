use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use core_money::{Satoshis, UsdCents};

use crate::{terms::InterestPeriod, CollateralizationState, CreditFacilityReceivable, TermValues};

use super::primitives::*;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCreditEvent {
    FacilityCreated {
        id: CreditFacilityId,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        terms: TermValues,
        amount: UsdCents,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        created_at: DateTime<Utc>,
    },
    FacilityApproved {
        id: CreditFacilityId,
    },
    FacilityActivated {
        id: CreditFacilityId,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        activation_tx_id: LedgerTxId,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        activated_at: DateTime<Utc>,
        amount: UsdCents,
    },
    FacilityCompleted {
        id: CreditFacilityId,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        completed_at: DateTime<Utc>,
    },
    FacilityRepaymentRecorded {
        credit_facility_id: CreditFacilityId,
        obligation_id: ObligationId,
        obligation_type: ObligationType,
        payment_id: PaymentAllocationId,
        amount: UsdCents,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        recorded_at: DateTime<Utc>,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        effective: chrono::NaiveDate,
    },
    FacilityCollateralUpdated {
        credit_facility_id: CreditFacilityId,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        ledger_tx_id: LedgerTxId,
        new_amount: Satoshis,
        abs_diff: Satoshis,
        action: CollateralAction,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        recorded_at: DateTime<Utc>,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        effective: chrono::NaiveDate,
    },
    FacilityCollateralizationChanged {
        id: CreditFacilityId,
        state: CollateralizationState,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        recorded_at: DateTime<Utc>,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        effective: chrono::NaiveDate,
        collateral: Satoshis,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        outstanding: CreditFacilityReceivable,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        price: PriceOfOneBTC,
    },
    DisbursalSettled {
        credit_facility_id: CreditFacilityId,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        ledger_tx_id: LedgerTxId,
        amount: UsdCents,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        recorded_at: DateTime<Utc>,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        effective: chrono::NaiveDate,
    },
    AccrualPosted {
        credit_facility_id: CreditFacilityId,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        ledger_tx_id: LedgerTxId,
        amount: UsdCents,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        period: InterestPeriod,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        recorded_at: DateTime<Utc>,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        effective: chrono::NaiveDate,
    },
    ObligationCreated {
        id: ObligationId,
        obligation_type: ObligationType,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,

        #[cfg_attr(feature = "schemars", schemars(skip))]
        due_at: DateTime<Utc>,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        overdue_at: Option<DateTime<Utc>>,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        defaulted_at: Option<DateTime<Utc>>,
        #[cfg_attr(feature = "schemars", schemars(skip))]
        recorded_at: DateTime<Utc>,
        #[cfg_attr(feature = "schemars", schemars(skip))]
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
}
