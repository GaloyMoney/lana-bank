use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoreCollectionsEvent {
    PaymentReceived {
        payment_id: PaymentId,
        facility_id: FacilityId,
        amount: UsdCents,
        source_account_id: CalaAccountId,
        ledger_tx_id: LedgerTxId,
        recorded_at: DateTime<Utc>,
    },
    PaymentAllocated {
        allocation_id: PaymentAllocationId,
        payment_id: PaymentId,
        facility_id: FacilityId,
        obligation_id: ObligationId,
        amount: UsdCents,
        ledger_tx_id: LedgerTxId,
        allocated_at: DateTime<Utc>,
    },
    ObligationCreated {
        obligation_id: ObligationId,
        facility_id: FacilityId,
        obligation_type: ObligationType,
        amount: UsdCents,
        created_at: DateTime<Utc>,
    },
    ObligationDue {
        obligation_id: ObligationId,
        facility_id: FacilityId,
        amount: UsdCents,
        due_at: DateTime<Utc>,
    },
    ObligationOverdue {
        obligation_id: ObligationId,
        facility_id: FacilityId,
        amount: UsdCents,
        overdue_at: DateTime<Utc>,
    },
    ObligationDefaulted {
        obligation_id: ObligationId,
        facility_id: FacilityId,
        amount: UsdCents,
        defaulted_at: DateTime<Utc>,
    },
    ObligationCompleted {
        obligation_id: ObligationId,
        facility_id: FacilityId,
        completed_at: DateTime<Utc>,
    },
}
