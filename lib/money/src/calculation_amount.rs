use rust_decimal::{Decimal, RoundingStrategy, prelude::ToPrimitive};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use crate::{Currency, MinorUnits, Precision, error::ConversionError};

/// High-precision monetary amount for intermediate financial calculations.
///
/// Wraps `Decimal` (28 significant digits) with currency type safety.
/// Arithmetic stays at full ~28dp precision; rounding is applied only at
/// explicit rounding boundaries (`round_to_precision`, `round_to_minor_units`).
///
/// Intentionally does NOT implement `Serialize`, `Deserialize`, `sqlx::Encode`,
/// `sqlx::Decode`, or GraphQL scalars — the only way to persist is through
/// explicit rounding.
#[derive(Clone, Copy)]
pub struct CalculationAmount<C: Currency> {
    value: Decimal,
    _currency: PhantomData<C>,
}

// ─── Construction ────────────────────────────────────────────────────

impl<C: Currency> CalculationAmount<C> {
    pub fn from_major(major: Decimal) -> Self {
        Self {
            value: major,
            _currency: PhantomData,
        }
    }

    pub fn from_minor(units: MinorUnits<C>) -> Self {
        Self::from_major(units.to_major())
    }

    pub fn zero() -> Self {
        Self {
            value: Decimal::ZERO,
            _currency: PhantomData,
        }
    }

    pub fn to_major(self) -> Decimal {
        self.value
    }

    pub fn to_minor_decimal(self) -> Decimal {
        self.value * Decimal::from(C::MINOR_UNITS_PER_MAJOR)
    }

    pub fn is_zero(self) -> bool {
        self.value.is_zero()
    }

    pub fn is_negative(self) -> bool {
        self.value.is_sign_negative() && !self.value.is_zero()
    }

    pub fn abs(self) -> Self {
        Self {
            value: self.value.abs(),
            ..self
        }
    }

    pub fn max(self, other: Self) -> Self {
        std::cmp::max(self, other)
    }

    pub fn min(self, other: Self) -> Self {
        std::cmp::min(self, other)
    }
}

// ─── The Rounding Boundary ──────────────────────────────────────────

impl<C: Currency> CalculationAmount<C> {
    /// Round to the given regulatory precision (e.g., 6dp) with the given strategy.
    /// For storage/reporting per NRP regulation.
    pub fn round_to_precision(&self, precision: Precision, strategy: RoundingStrategy) -> Decimal {
        self.value.round_dp_with_strategy(precision.dp(), strategy)
    }

    /// Round to the nearest minor unit using the given strategy.
    /// For ledger booking — caller chooses business-context strategy:
    /// - `AwayFromZero` — interest owed, fees, required collateral (lender-favorable)
    /// - `ToZero` — collateral valuation (conservative)
    ///
    /// # Panics
    ///
    /// Panics if the value is negative or exceeds `u64::MAX` minor units.
    pub fn round_to_minor_units(&self, strategy: RoundingStrategy) -> MinorUnits<C> {
        debug_assert!(
            !self.is_negative(),
            "CalculationAmount::round_to_minor_units called with negative value: {self}",
        );
        let minor = self.value * Decimal::from(C::MINOR_UNITS_PER_MAJOR);
        let rounded = minor.round_dp_with_strategy(0, strategy);
        MinorUnits::from(
            rounded
                .to_u64()
                .expect("CalculationAmount must be non-negative and within u64 range"),
        )
    }

    /// Fallible version of `round_to_minor_units`. Returns `Err` if the value
    /// is negative or exceeds `u64::MAX` minor units.
    pub fn try_round_to_minor_units(
        &self,
        strategy: RoundingStrategy,
    ) -> Result<MinorUnits<C>, ConversionError> {
        let minor = self.value * Decimal::from(C::MINOR_UNITS_PER_MAJOR);
        let rounded = minor.round_dp_with_strategy(0, strategy);
        rounded
            .to_u64()
            .map(MinorUnits::from)
            .ok_or(ConversionError::Overflow)
    }
}

// ─── Same-Currency Arithmetic (full precision, no auto-rounding) ────

impl<C: Currency> Add for CalculationAmount<C> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            value: self.value + rhs.value,
            ..self
        }
    }
}

impl<C: Currency> Sub for CalculationAmount<C> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            value: self.value - rhs.value,
            ..self
        }
    }
}

impl<C: Currency> AddAssign for CalculationAmount<C> {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
    }
}

impl<C: Currency> SubAssign for CalculationAmount<C> {
    fn sub_assign(&mut self, rhs: Self) {
        self.value -= rhs.value;
    }
}

impl<C: Currency> Neg for CalculationAmount<C> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value: -self.value,
            ..self
        }
    }
}

// ─── Scaling by Decimal ─────────────────────────────────────────────

impl<C: Currency> Mul<Decimal> for CalculationAmount<C> {
    type Output = Self;
    fn mul(self, rhs: Decimal) -> Self {
        Self {
            value: self.value * rhs,
            ..self
        }
    }
}

impl<C: Currency> Mul<CalculationAmount<C>> for Decimal {
    type Output = CalculationAmount<C>;
    fn mul(self, rhs: CalculationAmount<C>) -> CalculationAmount<C> {
        CalculationAmount {
            value: self * rhs.value,
            ..rhs
        }
    }
}

impl<C: Currency> Div<Decimal> for CalculationAmount<C> {
    type Output = Self;
    fn div(self, rhs: Decimal) -> Self {
        Self {
            value: self.value / rhs,
            ..self
        }
    }
}

impl<C: Currency> Div for CalculationAmount<C> {
    type Output = Decimal;
    fn div(self, rhs: Self) -> Decimal {
        self.value / rhs.value
    }
}

// ─── Debug & Display ────────────────────────────────────────────────

impl<C: Currency> fmt::Debug for CalculationAmount<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Calc<{}>({})", C::CODE, self.value)
    }
}

impl<C: Currency> fmt::Display for CalculationAmount<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "~{} {}", self.value, C::CODE)
    }
}

// ─── Ordering (compares value only) ─────────────────────────────────

impl<C: Currency> PartialEq for CalculationAmount<C> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<C: Currency> Eq for CalculationAmount<C> {}

impl<C: Currency> std::hash::Hash for CalculationAmount<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<C: Currency> PartialOrd for CalculationAmount<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Currency> Ord for CalculationAmount<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use rust_decimal::RoundingStrategy;
    use rust_decimal_macros::dec;

    use super::*;
    use crate::{Btc, Usd};

    type CalcUsd = CalculationAmount<Usd>;
    type CalcBtc = CalculationAmount<Btc>;

    #[test]
    fn zero_is_zero() {
        assert!(CalcUsd::zero().is_zero());
        assert!(!CalcUsd::from_major(dec!(1.5)).is_zero());
    }

    #[test]
    fn from_major_preserves_value() {
        let calc = CalcUsd::from_major(dec!(123.456789));
        assert_eq!(calc.to_major(), dec!(123.456789));
    }

    #[test]
    fn to_minor_decimal_converts_correctly() {
        let calc = CalcUsd::from_major(dec!(3.287671));
        assert_eq!(calc.to_minor_decimal(), dec!(328.7671));
    }

    #[test]
    fn to_minor_decimal_btc() {
        let calc = CalcBtc::from_major(dec!(0.00166667));
        assert_eq!(calc.to_minor_decimal(), dec!(166667.00000000));
    }

    #[test]
    fn add_same_currency() {
        let a = CalcUsd::from_major(dec!(1.50));
        let b = CalcUsd::from_major(dec!(2.75));
        assert_eq!((a + b).to_major(), dec!(4.25));
    }

    #[test]
    fn sub_same_currency() {
        let a = CalcUsd::from_major(dec!(5.00));
        let b = CalcUsd::from_major(dec!(2.25));
        assert_eq!((a - b).to_major(), dec!(2.75));
    }

    #[test]
    fn mul_by_decimal() {
        let amt = CalcUsd::from_major(dec!(100.00));
        let rate = dec!(0.12);
        assert_eq!((amt * rate).to_major(), dec!(12.00));
    }

    #[test]
    fn decimal_mul_calc() {
        let amt = CalcUsd::from_major(dec!(100.00));
        let rate = dec!(0.12);
        assert_eq!((rate * amt).to_major(), dec!(12.00));
    }

    #[test]
    fn div_by_decimal() {
        let amt = CalcUsd::from_major(dec!(12.00));
        let divisor = dec!(365);
        let expected = dec!(12.00) / dec!(365);
        assert_eq!((amt / divisor).to_major(), expected);
    }

    #[test]
    fn div_calc_by_calc_returns_decimal() {
        let a = CalcUsd::from_major(dec!(10.00));
        let b = CalcUsd::from_major(dec!(4.00));
        let ratio: Decimal = a / b;
        assert_eq!(ratio, dec!(2.5));
    }

    #[test]
    fn neg() {
        let a = CalcUsd::from_major(dec!(5.00));
        assert_eq!((-a).to_major(), dec!(-5.00));
    }

    #[test]
    fn add_assign() {
        let mut a = CalcUsd::from_major(dec!(1.00));
        a += CalcUsd::from_major(dec!(2.50));
        assert_eq!(a.to_major(), dec!(3.50));
    }

    #[test]
    fn sub_assign() {
        let mut a = CalcUsd::from_major(dec!(5.00));
        a -= CalcUsd::from_major(dec!(2.00));
        assert_eq!(a.to_major(), dec!(3.00));
    }

    #[test]
    fn fold_summation() {
        let amounts = [
            CalcUsd::from_major(dec!(1.10)),
            CalcUsd::from_major(dec!(2.20)),
            CalcUsd::from_major(dec!(3.30)),
        ];
        let total = amounts.into_iter().fold(CalcUsd::zero(), |acc, x| acc + x);
        assert_eq!(total.to_major(), dec!(6.60));
    }

    #[test]
    fn is_negative() {
        assert!(CalcUsd::from_major(dec!(-1.00)).is_negative());
        assert!(!CalcUsd::from_major(dec!(1.00)).is_negative());
        assert!(!CalcUsd::zero().is_negative());
    }

    #[test]
    fn abs() {
        assert_eq!(
            CalcUsd::from_major(dec!(-5.00)).abs().to_major(),
            dec!(5.00)
        );
        assert_eq!(CalcUsd::from_major(dec!(5.00)).abs().to_major(), dec!(5.00));
    }

    #[test]
    fn max_and_min() {
        let a = CalcUsd::from_major(dec!(3.00));
        let b = CalcUsd::from_major(dec!(7.00));
        assert_eq!(a.max(b).to_major(), dec!(7.00));
        assert_eq!(a.min(b).to_major(), dec!(3.00));
    }

    #[test]
    fn debug_format() {
        let a = CalcUsd::from_major(dec!(1.23));
        assert_eq!(format!("{:?}", a), "Calc<USD>(1.23)");
    }

    #[test]
    fn display_format() {
        let a = CalcUsd::from_major(dec!(1.23));
        assert_eq!(format!("{}", a), "~1.23 USD");
    }

    #[test]
    fn ordering() {
        let a = CalcUsd::from_major(dec!(1.00));
        let b = CalcUsd::from_major(dec!(2.00));
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, CalcUsd::from_major(dec!(1.00)));
    }

    #[test]
    fn from_minor_units() {
        let cents = MinorUnits::<Usd>::from(12345u64);
        let calc = CalculationAmount::from_minor(cents);
        assert_eq!(calc.to_major(), dec!(123.45));
    }

    #[test]
    fn round_to_minor_units_away_from_zero() {
        let calc = CalcUsd::from_major(dec!(3.287671));
        let rounded = calc.round_to_minor_units(RoundingStrategy::AwayFromZero);
        assert_eq!(rounded.into_inner(), 329);
    }

    #[test]
    fn round_to_minor_units_to_zero() {
        let calc = CalcUsd::from_major(dec!(3.287671));
        let rounded = calc.round_to_minor_units(RoundingStrategy::ToZero);
        assert_eq!(rounded.into_inner(), 328);
    }

    #[test]
    fn round_to_minor_units_btc() {
        let calc = CalcBtc::from_major(dec!(0.001666666666));
        let rounded = calc.round_to_minor_units(RoundingStrategy::AwayFromZero);
        assert_eq!(rounded.into_inner(), 166667);
    }

    #[test]
    fn round_trip_lossless() {
        let original = MinorUnits::<Usd>::from(329u64);
        let calc = CalculationAmount::from_minor(original);
        let back = calc.round_to_minor_units(RoundingStrategy::AwayFromZero);
        assert_eq!(original, back);
    }

    #[test]
    fn round_zero() {
        let calc = CalcUsd::zero();
        assert_eq!(
            calc.round_to_minor_units(RoundingStrategy::AwayFromZero)
                .into_inner(),
            0
        );
        assert_eq!(
            calc.round_to_minor_units(RoundingStrategy::ToZero)
                .into_inner(),
            0
        );
    }

    #[test]
    fn round_to_precision() {
        let precision = Precision::try_new(6).unwrap();
        let strategy = RoundingStrategy::MidpointAwayFromZero;

        // 3.287671 USD rounded to 6dp with MidpointAwayFromZero → 3.287671 (already 6dp)
        let calc = CalcUsd::from_major(dec!(3.287671));
        assert_eq!(calc.round_to_precision(precision, strategy), dec!(3.287671));

        // Higher precision value rounded to 6dp
        let calc2 = CalcUsd::from_major(dec!(3.2876714999));
        assert_eq!(
            calc2.round_to_precision(precision, strategy),
            dec!(3.287671)
        );

        // Midpoint rounds up with MidpointAwayFromZero
        let calc3 = CalcUsd::from_major(dec!(3.2876715));
        assert_eq!(
            calc3.round_to_precision(precision, strategy),
            dec!(3.287672)
        );
    }

    #[test]
    fn arithmetic_preserves_full_precision() {
        let principal = CalcUsd::from_major(dec!(10000));
        let rate = dec!(0.12);
        let days_in_year = dec!(365);

        // Single daily interest at full precision
        let daily = principal * rate / days_in_year;
        // Full precision: 3.28767123287671232876712328...
        assert_eq!(daily.to_major(), dec!(10000) * dec!(0.12) / dec!(365));

        // Sum 30 days — should maintain full precision
        let total = (0..30).fold(CalcUsd::zero(), |acc, _| acc + daily);
        let sum_then_round = total.round_to_minor_units(RoundingStrategy::AwayFromZero);

        // Round each day then sum (the wrong way)
        let round_then_sum: MinorUnits<Usd> = (0..30)
            .map(|_| daily.round_to_minor_units(RoundingStrategy::AwayFromZero))
            .sum();

        // sum-then-round should be <= round-then-sum (no precision loss)
        assert!(sum_then_round <= round_then_sum);
    }
}
