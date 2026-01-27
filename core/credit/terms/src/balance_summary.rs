use serde::{Deserialize, Serialize};

pub use super::{collateralization::CollateralizationRatio, cvl::*};
pub use core_money::*;
pub use core_price::*;

#[cfg(not(test))]
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CreditFacilityBalanceSummary {
    pub(super) facility: UsdCents,
    pub(super) facility_remaining: UsdCents,
    pub(super) collateral: Satoshis,
    pub(super) disbursed: UsdCents,
    pub(super) not_yet_due_disbursed_outstanding: UsdCents,
    pub(super) due_disbursed_outstanding: UsdCents,
    pub(super) overdue_disbursed_outstanding: UsdCents,
    pub(super) disbursed_defaulted: UsdCents,
    pub(super) interest_posted: UsdCents,
    pub(super) not_yet_due_interest_outstanding: UsdCents,
    pub(super) due_interest_outstanding: UsdCents,
    pub(super) overdue_interest_outstanding: UsdCents,
    pub(super) interest_defaulted: UsdCents,
    pub(super) payments_unapplied: UsdCents,
}

// For testing we want to be able to construct the struct
#[cfg(test)]
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CreditFacilityBalanceSummary {
    pub facility: UsdCents,
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
    pub payments_unapplied: UsdCents,
}

impl CreditFacilityBalanceSummary {
    pub fn any_disbursed(&self) -> bool {
        !self.disbursed.is_zero()
    }

    pub fn facility(&self) -> UsdCents {
        self.facility
    }

    pub fn facility_remaining(&self) -> UsdCents {
        self.facility_remaining
    }

    pub fn overdue_disbursed_outstanding(&self) -> UsdCents {
        self.overdue_disbursed_outstanding
    }

    pub fn disbursed_outstanding_payable(&self) -> UsdCents {
        self.due_disbursed_outstanding + self.overdue_disbursed_outstanding
    }

    pub fn disbursed_outstanding(&self) -> UsdCents {
        self.not_yet_due_disbursed_outstanding + self.disbursed_outstanding_payable()
    }

    pub fn overdue_interest_outstanding(&self) -> UsdCents {
        self.overdue_interest_outstanding
    }

    pub fn interest_outstanding_payable(&self) -> UsdCents {
        self.due_interest_outstanding + self.overdue_interest_outstanding
    }

    pub fn interest_outstanding(&self) -> UsdCents {
        self.not_yet_due_interest_outstanding + self.interest_outstanding_payable()
    }

    pub fn total_outstanding(&self) -> UsdCents {
        self.disbursed_outstanding() + self.interest_outstanding()
    }

    pub fn interest_posted(&self) -> UsdCents {
        self.interest_posted
    }

    pub fn collateral(&self) -> Satoshis {
        self.collateral
    }

    pub fn payments_unapplied(&self) -> UsdCents {
        self.payments_unapplied
    }

    pub fn total_outstanding_payable(&self) -> UsdCents {
        self.disbursed_outstanding_payable() + self.interest_outstanding_payable()
    }

    fn total_outstanding_not_yet_payable(&self) -> UsdCents {
        self.not_yet_due_disbursed_outstanding + self.not_yet_due_interest_outstanding
    }

    pub fn total_disbursed(&self) -> UsdCents {
        self.disbursed
    }

    pub fn total_overdue(&self) -> UsdCents {
        self.overdue_disbursed_outstanding + self.overdue_interest_outstanding
    }

    fn total_defaulted(&self) -> UsdCents {
        self.disbursed_defaulted + self.interest_defaulted
    }

    pub fn any_outstanding_or_defaulted(&self) -> bool {
        !(self.total_outstanding_not_yet_payable().is_zero()
            && self.total_outstanding_payable().is_zero()
            && self.total_defaulted().is_zero())
    }

    pub fn with_collateral(self, collateral: Satoshis) -> Self {
        Self { collateral, ..self }
    }

    pub fn with_added_disbursal(self, disbursal: UsdCents) -> Self {
        Self {
            disbursed: self.disbursed + disbursal,
            not_yet_due_disbursed_outstanding: self.not_yet_due_disbursed_outstanding + disbursal,
            ..self
        }
    }

    pub fn current_collateralization_ratio(&self) -> CollateralizationRatio {
        CVLData::new(self.collateral(), self.total_outstanding()).ratio()
    }

    pub fn current_cvl(&self, price: PriceOfOneBTC) -> CVLPct {
        CVLData::new(self.collateral(), self.total_outstanding()).cvl(price)
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PendingCreditFacilityBalanceSummary {
    facility: UsdCents,
    collateral: Satoshis,
}

impl PendingCreditFacilityBalanceSummary {
    pub fn new(facility: UsdCents, collateral: Satoshis) -> Self {
        Self {
            collateral,
            facility,
        }
    }

    pub fn collateral(&self) -> Satoshis {
        self.collateral
    }

    pub fn current_collateralization_ratio(&self) -> CollateralizationRatio {
        CVLData::new(self.collateral, self.facility).ratio()
    }

    pub fn current_cvl(&self, price: PriceOfOneBTC) -> CVLPct {
        CVLData::new(self.collateral, self.facility).cvl(price)
    }
}
