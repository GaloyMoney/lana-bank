use serde::{Deserialize, Serialize};

use core_money::{Satoshis, UsdCents};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityBalanceSummary {
    pub facility_remaining: UsdCents,
    pub collateral: Satoshis,
    pub total_disbursed: UsdCents,
    pub disbursed_receivable: UsdCents,
    // pub due_disbursed_receivable: UsdCents,
    // pub overdue_disbursed_receivable: UsdCents,
    pub total_interest_accrued: UsdCents,
    pub interest_receivable: UsdCents,
    // pub due_interest_receivable: UsdCents,
    // pub overdue_interest_receivable: UsdCents,
}

impl CreditFacilityBalanceSummary {
    pub fn any_disbursed(&self) -> bool {
        self.total_disbursed > UsdCents::ZERO
    }

    pub fn any_outstanding(&self) -> bool {
        self.disbursed_receivable > UsdCents::ZERO || self.interest_receivable > UsdCents::ZERO
    }

    pub fn total_outstanding(&self) -> UsdCents {
        self.disbursed_receivable + self.interest_receivable
    }

    pub fn total_overdue(&self) -> UsdCents {
        // self.overdue_disbursed_receivable + self.overdue_interest_receivable
        unimplemented!()
    }
}
