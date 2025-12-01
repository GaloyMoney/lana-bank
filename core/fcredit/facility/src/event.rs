use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_money::UsdCents;
use credit_terms::TermValues;

use super::primitives::*;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCreditFacilityEvent {
    FacilityProposalCreated {
        id: CreditFacilityProposalId,
        terms: TermValues,
        amount: UsdCents,
        created_at: DateTime<Utc>,
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
}
