use serde::{Deserialize, Serialize};

use core_money::{Satoshis, UsdCents};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityBalanceSummary {
    pub facility_remaining: UsdCents,
    pub collateral: Satoshis,
    pub total_disbursed: UsdCents,
    pub disbursed_receivable: UsdCents,
    // pub due_disbursed_receivable: UsdCents,
    pub total_interest_accrued: UsdCents,
    pub interest_receivable: UsdCents,
    // pub due_interest_receivable: UsdCents,
}
