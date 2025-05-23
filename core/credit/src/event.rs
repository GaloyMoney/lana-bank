use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use core_money::{Satoshis, UsdCents};

use crate::{terms::InterestPeriod, CollateralizationState, CreditFacilityReceivable, TermValues};

use super::primitives::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreCreditEvent {
    FacilityCreated {
        id: CreditFacilityId,
        terms: TermValues,
        amount: UsdCents,
        created_at: DateTime<Utc>,
    },
    FacilityApproved {
        id: CreditFacilityId,
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
    },
    FacilityCollateralUpdated {
        credit_facility_id: CreditFacilityId,
        ledger_tx_id: LedgerTxId,
        new_amount: Satoshis,
        abs_diff: Satoshis,
        action: CollateralAction,
        recorded_at: DateTime<Utc>,
    },
    FacilityCollateralizationChanged {
        id: CreditFacilityId,
        state: CollateralizationState,
        recorded_at: DateTime<Utc>,
        collateral: Satoshis,
        outstanding: CreditFacilityReceivable,
        price: PriceOfOneBTC,
    },
    DisbursalSettled {
        credit_facility_id: CreditFacilityId,
        ledger_tx_id: LedgerTxId,
        amount: UsdCents,
        recorded_at: DateTime<Utc>,
    },
    AccrualPosted {
        credit_facility_id: CreditFacilityId,
        ledger_tx_id: LedgerTxId,
        amount: UsdCents,
        period: InterestPeriod,
        recorded_at: DateTime<Utc>,
    },
    ObligationCreated {
        id: ObligationId,
        obligation_type: ObligationType,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,

        due_at: DateTime<Utc>,
        overdue_at: Option<DateTime<Utc>>,
        defaulted_at: Option<DateTime<Utc>>,
        recorded_at: DateTime<Utc>,
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
