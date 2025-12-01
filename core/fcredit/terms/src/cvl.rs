use rust_decimal::{Decimal, prelude::*};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_money::{Satoshis, UsdCents};
use core_price::PriceOfOneBTC;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CVLPct {
    Finite(Decimal),
    Infinite,
}

#[cfg(feature = "graphql")]
async_graphql::scalar!(CVLPct);

impl PartialOrd for CVLPct {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match (self, other) {
            (Self::Infinite, Self::Infinite) => Some(Ordering::Equal),
            (Self::Infinite, Self::Finite(_)) => Some(Ordering::Greater),
            (Self::Finite(_), Self::Infinite) => Some(Ordering::Less),
            (Self::Finite(a), Self::Finite(b)) => a.partial_cmp(b),
        }
    }
}

impl CVLPct {
    pub const ZERO: Self = Self::Finite(dec!(0));
    pub const UPGRADE_BUFFER: Self = Self::Finite(dec!(5));

    pub fn new(value: u64) -> Self {
        Self::Finite(Decimal::from(value))
    }

    pub fn from_loan_amounts(
        collateral_value: UsdCents,
        total_outstanding_amount: UsdCents,
    ) -> Self {
        if collateral_value.is_zero() {
            return Self::ZERO;
        }

        if total_outstanding_amount.is_zero() {
            return Self::Infinite;
        }

        let ratio = (collateral_value.to_usd() / total_outstanding_amount.to_usd())
            .round_dp_with_strategy(2, RoundingStrategy::ToZero)
            * dec!(100);

        CVLPct::Finite(ratio)
    }

    pub fn is_zero(&self) -> bool {
        matches!(self, Self::Finite(value) if *value == dec!(0))
    }

    pub fn scale(&self, value: UsdCents) -> UsdCents {
        match self {
            Self::Finite(pct) => {
                let cents = value.to_usd() * dec!(100) * (pct / dec!(100));
                UsdCents::from(
                    cents
                        .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero)
                        .to_u64()
                        .expect("should return a valid integer"),
                )
            }
            Self::Infinite => unreachable!("Cannot scale with infinite CVL percentage"),
        }
    }

    pub fn is_significantly_lower_than(&self, other: CVLPct, buffer: CVLPct) -> bool {
        other > *self + buffer
    }

    #[cfg(test)]
    pub fn target_value_given_outstanding(&self, outstanding: UsdCents) -> UsdCents {
        match self {
            Self::Finite(pct) => {
                let target_in_usd = pct / dec!(100) * outstanding.to_usd();
                UsdCents::from(
                    (target_in_usd * dec!(100))
                        .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero)
                        .to_u64()
                        .expect("should return a valid integer"),
                )
            }
            Self::Infinite => {
                unreachable!("Cannot calculate target value for infinite CVL percentage")
            }
        }
    }
}

impl fmt::Display for CVLPct {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        match self {
            Self::Finite(value) => write!(f, "{value}"),
            Self::Infinite => write!(f, "∞"),
        }
    }
}

impl std::ops::Add for CVLPct {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => Self::Finite(a + b),
            _ => Self::Infinite,
        }
    }
}

#[cfg(test)]
impl std::ops::Sub for CVLPct {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Self::Infinite, Self::Finite(_)) => Self::Infinite,
            (Self::Finite(_), Self::Infinite) => panic!("Cannot subtract infinite from finite"),
            (Self::Infinite, Self::Infinite) => panic!("Infinite - Infinite is undefined"),
            (Self::Finite(a), Self::Finite(b)) => Self::Finite(a - b),
        }
    }
}

impl From<Decimal> for CVLPct {
    fn from(value: Decimal) -> Self {
        CVLPct::Finite(value)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CVLData {
    amount: UsdCents,
    collateral: Satoshis,
}

impl CVLData {
    pub(crate) fn new(collateral: Satoshis, amount: UsdCents) -> Self {
        Self { collateral, amount }
    }

    pub(crate) fn cvl(&self, price: PriceOfOneBTC) -> CVLPct {
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
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn loan_cvl_pct_scale() {
        let cvl = CVLPct::Finite(dec!(140));
        let value = UsdCents::from(100000);
        let scaled = cvl.scale(value);
        assert_eq!(scaled, UsdCents::try_from_usd(dec!(1400)).unwrap());

        let cvl = CVLPct::Finite(dec!(50));
        let value = UsdCents::from(333333);
        let scaled = cvl.scale(value);
        assert_eq!(scaled, UsdCents::try_from_usd(dec!(1666.67)).unwrap());
    }

    #[test]
    fn current_cvl_from_loan_amounts() {
        let expected_cvl = CVLPct::Finite(dec!(125));
        let collateral_value = UsdCents::from(125000);
        let outstanding_amount = UsdCents::from(100000);
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(cvl, expected_cvl);

        let expected_cvl = CVLPct::Finite(dec!(75));
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

        let expected_cvl = CVLPct::Infinite;
        let collateral_value = UsdCents::from(75000);
        let outstanding_amount = UsdCents::ZERO;
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(cvl, expected_cvl);

        let expected_cvl = CVLPct::ZERO;
        let collateral_value = UsdCents::ZERO;
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
