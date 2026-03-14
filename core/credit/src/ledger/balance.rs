use core_price::PriceOfOneBTC;
use rust_decimal::Decimal;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use money::{Satoshis, UsdCents};

use crate::{CVLPct, CollateralizationRatio};

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

#[derive(Clone, Debug)]
struct CVLData {
    amount: UsdCents,
    collateral: Satoshis,
}

impl CVLData {
    fn new(collateral: Satoshis, amount: UsdCents) -> Self {
        Self { collateral, amount }
    }

    fn ratio(&self) -> CollateralizationRatio {
        if self.amount.is_zero() {
            return CollateralizationRatio::Infinite;
        }

        let amount = Decimal::from(self.amount.into_inner());
        let collateral = Decimal::from(self.collateral.into_inner());

        CollateralizationRatio::Finite(collateral / amount)
    }

    fn cvl(&self, price: PriceOfOneBTC) -> CVLPct {
        let collateral_value = price.sats_to_cents_round_down(self.collateral);
        if collateral_value == UsdCents::ZERO {
            CVLPct::ZERO
        } else {
            CVLPct::from_loan_amounts(collateral_value, self.amount)
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::inconsistent_digit_grouping)]

    use super::*;

    #[test]
    fn current_cvl_returns_infinite_when_no_disbursals() {
        let balances = CreditFacilityBalanceSummary {
            collateral: Satoshis::from(100),
            facility: UsdCents::from(2),
            disbursed: UsdCents::ZERO,

            not_yet_due_disbursed_outstanding: UsdCents::ZERO,
            due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,

            facility_remaining: UsdCents::from(1),
            interest_posted: UsdCents::from(1),
            payments_unapplied: UsdCents::ZERO,
        };

        let price = PriceOfOneBTC::new(UsdCents::from(100_000_00));
        assert_eq!(balances.current_cvl(price), CVLPct::Infinite);
    }

    #[test]
    fn current_cvl_returns_non_zero_amount_when_disbursals_with_outstanding() {
        let balances = CreditFacilityBalanceSummary {
            collateral: Satoshis::from(100),
            facility: UsdCents::from(2),
            disbursed: UsdCents::from(1),

            not_yet_due_disbursed_outstanding: UsdCents::ONE,
            due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,

            facility_remaining: UsdCents::from(1),
            interest_posted: UsdCents::from(1),
            payments_unapplied: UsdCents::ZERO,
        };

        let price = PriceOfOneBTC::new(UsdCents::from(100_000_00));
        assert_ne!(balances.current_cvl(price), CVLPct::ZERO);
        assert_ne!(balances.current_cvl(price), CVLPct::Infinite);
    }

    #[test]
    fn current_cvl_returns_infinite_when_disbursals_with_no_outstanding() {
        let balances = CreditFacilityBalanceSummary {
            collateral: Satoshis::from(100),
            facility: UsdCents::from(2),
            disbursed: UsdCents::from(1),

            not_yet_due_disbursed_outstanding: UsdCents::ZERO,
            due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,

            facility_remaining: UsdCents::from(1),
            interest_posted: UsdCents::from(1),
            payments_unapplied: UsdCents::ZERO,
        };

        let price = PriceOfOneBTC::new(UsdCents::from(100_000_00));
        assert_eq!(balances.current_cvl(price), CVLPct::Infinite);
    }

    #[test]
    fn current_collateralization_ratio_when_no_disbursals() {
        let balances = CreditFacilityBalanceSummary {
            collateral: Satoshis::from(100),
            facility: UsdCents::from(2),
            disbursed: UsdCents::ZERO,
            due_disbursed_outstanding: UsdCents::ZERO,

            not_yet_due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,

            facility_remaining: UsdCents::from(1),
            interest_posted: UsdCents::from(1),
            payments_unapplied: UsdCents::ZERO,
        };

        assert_eq!(
            balances.current_collateralization_ratio(),
            CollateralizationRatio::Infinite
        );
    }

    #[test]
    fn current_collateralization_ratio_when_disbursals() {
        let balances = CreditFacilityBalanceSummary {
            collateral: Satoshis::from(100),
            facility: UsdCents::from(2),
            disbursed: UsdCents::from(1),
            due_disbursed_outstanding: UsdCents::from(1),

            not_yet_due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,

            facility_remaining: UsdCents::from(1),
            interest_posted: UsdCents::from(1),
            payments_unapplied: UsdCents::ZERO,
        };

        let collateral = Decimal::from(balances.collateral().into_inner());
        let expected =
            collateral / Decimal::from(balances.total_outstanding_payable().into_inner());
        assert_eq!(
            balances.current_collateralization_ratio(),
            CollateralizationRatio::Finite(expected)
        );
    }
}
