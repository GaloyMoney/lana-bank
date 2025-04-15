use serde::{Deserialize, Serialize};

use core_money::{Satoshis, UsdCents};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityBalanceSummary {
    pub facility_remaining: UsdCents,
    pub collateral: Satoshis,
    pub disbursed: UsdCents,
    pub not_yet_due_disbursed_outstanding: UsdCents,
    pub due_disbursed_outstanding: UsdCents,
    pub overdue_disbursed_outstanding: UsdCents,
    pub disbursed_defaulted: UsdCents,
    pub interest_posted: UsdCents,
    pub not_yet_due_interest_outstanding: UsdCents,
    pub due_interest_outstanding: UsdCents,
    pub overdue_interest_outstanding: UsdCents,
    pub interest_defaulted: UsdCents,
}

impl CreditFacilityBalanceSummary {
    pub fn any_disbursed(&self) -> bool {
        !self.disbursed.is_zero()
    }

    pub fn disbursed_outstanding_payable(&self) -> UsdCents {
        self.due_disbursed_outstanding + self.overdue_disbursed_outstanding
    }

    pub fn interest_outstanding_payable(&self) -> UsdCents {
        self.due_interest_outstanding + self.overdue_interest_outstanding
    }

    pub fn total_outstanding_payable(&self) -> UsdCents {
        self.disbursed_outstanding_payable() + self.interest_outstanding_payable()
    }

    fn total_not_yet_due(&self) -> UsdCents {
        self.not_yet_due_disbursed_outstanding + self.not_yet_due_interest_outstanding
    }

    pub fn total_overdue(&self) -> UsdCents {
        self.overdue_disbursed_outstanding + self.overdue_interest_outstanding
    }

    fn total_defaulted(&self) -> UsdCents {
        self.disbursed_defaulted + self.interest_defaulted
    }

    pub fn any_outstanding_or_defaulted(&self) -> bool {
        !(self.total_not_yet_due().is_zero()
            && self.total_outstanding_payable().is_zero()
            && self.total_defaulted().is_zero())
    }
}
