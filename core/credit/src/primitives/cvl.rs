use rust_decimal::{Decimal, prelude::*};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use core_money::UsdCents;

use std::fmt;

#[cfg_attr(feature = "graphql", derive(async_graphql::SimpleObject))]
pub struct FacilityCVL {
    pub total: CVLPct,
    pub disbursed: CVLPct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CVLPct(Decimal);
#[cfg(feature = "graphql")]
async_graphql::scalar!(CVLPct);

impl CVLPct {
    pub const ZERO: Self = Self(dec!(0));

    pub fn new(value: u64) -> Self {
        Self(Decimal::from(value))
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    pub fn scale(&self, value: UsdCents) -> UsdCents {
        let cents = value.to_usd() * dec!(100) * (self.0 / dec!(100));
        UsdCents::from(
            cents
                .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero)
                .to_u64()
                .expect("should return a valid integer"),
        )
    }

    pub fn from_loan_amounts(
        collateral_value: UsdCents,
        total_outstanding_amount: UsdCents,
    ) -> Self {
        if collateral_value == UsdCents::ZERO || total_outstanding_amount == UsdCents::ZERO {
            return CVLPct::ZERO;
        }

        let ratio = (collateral_value.to_usd() / total_outstanding_amount.to_usd())
            .round_dp_with_strategy(2, RoundingStrategy::ToZero)
            * dec!(100);

        CVLPct::from(ratio)
    }

    pub fn is_significantly_lower_than(&self, other: CVLPct, buffer: CVLPct) -> bool {
        other > *self + buffer
    }

    #[cfg(test)]
    pub fn target_value_given_outstanding(&self, outstanding: UsdCents) -> UsdCents {
        let target_in_usd = self.0 / dec!(100) * outstanding.to_usd();
        UsdCents::from(
            (target_in_usd * dec!(100))
                .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero)
                .to_u64()
                .expect("should return a valid integer"),
        )
    }
}

impl fmt::Display for CVLPct {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add for CVLPct {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        CVLPct(self.0 + other.0)
    }
}

impl std::ops::Sub for CVLPct {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        CVLPct(self.0 - other.0)
    }
}

impl From<Decimal> for CVLPct {
    fn from(value: Decimal) -> Self {
        CVLPct(value)
    }
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn loan_cvl_pct_scale() {
        let cvl = CVLPct(dec!(140));
        let value = UsdCents::from(100000);
        let scaled = cvl.scale(value);
        assert_eq!(scaled, UsdCents::try_from_usd(dec!(1400)).unwrap());

        let cvl = CVLPct(dec!(50));
        let value = UsdCents::from(333333);
        let scaled = cvl.scale(value);
        assert_eq!(scaled, UsdCents::try_from_usd(dec!(1666.67)).unwrap());
    }

    #[test]
    fn current_cvl_from_loan_amounts() {
        let expected_cvl = CVLPct(dec!(125));
        let collateral_value = UsdCents::from(125000);
        let outstanding_amount = UsdCents::from(100000);
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(cvl, expected_cvl);

        let expected_cvl = CVLPct(dec!(75));
        let collateral_value = UsdCents::from(75000);
        let outstanding_amount = UsdCents::from(100000);
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(cvl, expected_cvl);
    }

    #[test]
    fn current_cvl_for_zero_amounts() {
        let expected_cvl = CVLPct::ZERO;
        let collateral_value = UsdCents::ZERO;
        let outstanding_amount = UsdCents::from(100000);
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(cvl, expected_cvl);

        let expected_cvl = CVLPct::ZERO;
        let collateral_value = UsdCents::from(75000);
        let outstanding_amount = UsdCents::ZERO;
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(cvl, expected_cvl);
    }

    #[test]
    fn cvl_is_significantly_higher() {
        let buffer = CVLPct::new(5);

        let collateral_value = UsdCents::from(125000);
        let outstanding_amount = UsdCents::from(100000);
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        let collateral_value = UsdCents::from(130999);
        let outstanding_amount = UsdCents::from(100000);
        let slightly_higher_cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert!(!cvl.is_significantly_lower_than(slightly_higher_cvl, buffer));
        let collateral_value = UsdCents::from(131000);
        let outstanding_amount = UsdCents::from(100000);
        let significantly_higher_cvl =
            CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert!(cvl.is_significantly_lower_than(significantly_higher_cvl, buffer));
    }
}
