use rust_decimal::{Decimal, RoundingStrategy, prelude::ToPrimitive};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use crate::{Currency, MinorUnits, SignedMinorUnits};

/// High-precision monetary amount for intermediate financial calculations.
///
/// Wraps `Decimal` (28 significant digits) with currency type safety.
/// Intentionally does NOT implement `Serialize`, `Deserialize`, `sqlx::Encode`,
/// `sqlx::Decode`, or GraphQL scalars — the only way to persist is through
/// explicit rounding to `MinorUnits<C>`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CalculationAmount<C: Currency> {
    value: Decimal,
    _currency: PhantomData<C>,
}

// ─── Construction ────────────────────────────────────────────────────

impl<C: Currency> CalculationAmount<C> {
    pub const ZERO: Self = Self {
        value: Decimal::ZERO,
        _currency: PhantomData,
    };

    pub fn from_major(major: Decimal) -> Self {
        Self {
            value: major,
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
        Self::from_major(self.value.abs())
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
    /// Round to `MinorUnits<C>` using the specified strategy.
    ///
    /// # Panics
    /// Panics if the result is negative or exceeds `u64::MAX`.
    pub fn round_with(self, strategy: RoundingStrategy) -> MinorUnits<C> {
        let minor = self.value * Decimal::from(C::MINOR_UNITS_PER_MAJOR);
        let rounded = minor.round_dp_with_strategy(0, strategy);
        MinorUnits::from(
            rounded
                .to_u64()
                .expect("CalculationAmount must be non-negative and within u64 range"),
        )
    }

    /// Round away from zero (UP for positive amounts).
    /// Use for: interest owed, fees, required collateral, repay amounts.
    pub fn round_up(self) -> MinorUnits<C> {
        self.round_with(RoundingStrategy::AwayFromZero)
    }

    /// Round toward zero (DOWN for positive amounts).
    /// Use for: collateral valuation, CVL ratios.
    pub fn round_down(self) -> MinorUnits<C> {
        self.round_with(RoundingStrategy::ToZero)
    }

    /// Round half-to-even (banker's rounding).
    pub fn round_bankers(self) -> MinorUnits<C> {
        self.round_with(RoundingStrategy::MidpointNearestEven)
    }

    /// Round to `SignedMinorUnits<C>` (for values that may be negative).
    pub fn round_to_signed(self, strategy: RoundingStrategy) -> SignedMinorUnits<C> {
        let minor = self.value * Decimal::from(C::MINOR_UNITS_PER_MAJOR);
        let rounded = minor.round_dp_with_strategy(0, strategy);
        SignedMinorUnits::try_from_major(rounded / Decimal::from(C::MINOR_UNITS_PER_MAJOR))
            .expect("rounded value should convert cleanly")
    }
}

// ─── Lossless Widening: From<MinorUnits<C>> ─────────────────────────

impl<C: Currency> From<MinorUnits<C>> for CalculationAmount<C> {
    fn from(units: MinorUnits<C>) -> Self {
        Self {
            value: units.to_major(),
            _currency: PhantomData,
        }
    }
}

impl<C: Currency> From<SignedMinorUnits<C>> for CalculationAmount<C> {
    fn from(units: SignedMinorUnits<C>) -> Self {
        Self {
            value: units.to_major(),
            _currency: PhantomData,
        }
    }
}

// ─── Same-Currency Arithmetic ───────────────────────────────────────

impl<C: Currency> Add for CalculationAmount<C> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::from_major(self.value + rhs.value)
    }
}

impl<C: Currency> Sub for CalculationAmount<C> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::from_major(self.value - rhs.value)
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
        Self::from_major(-self.value)
    }
}

// ─── Scaling by Decimal ─────────────────────────────────────────────

impl<C: Currency> Mul<Decimal> for CalculationAmount<C> {
    type Output = Self;
    fn mul(self, rhs: Decimal) -> Self {
        Self::from_major(self.value * rhs)
    }
}

impl<C: Currency> Mul<CalculationAmount<C>> for Decimal {
    type Output = CalculationAmount<C>;
    fn mul(self, rhs: CalculationAmount<C>) -> CalculationAmount<C> {
        CalculationAmount::from_major(self * rhs.value)
    }
}

impl<C: Currency> Div<Decimal> for CalculationAmount<C> {
    type Output = Self;
    fn div(self, rhs: Decimal) -> Self {
        Self::from_major(self.value / rhs)
    }
}

impl<C: Currency> Div for CalculationAmount<C> {
    type Output = Decimal;
    fn div(self, rhs: Self) -> Decimal {
        self.value / rhs.value
    }
}

// ─── Iterator Support ───────────────────────────────────────────────

impl<C: Currency> std::iter::Sum for CalculationAmount<C> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + x)
    }
}

impl<'a, C: Currency> std::iter::Sum<&'a CalculationAmount<C>> for CalculationAmount<C> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + *x)
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

// ─── Default ────────────────────────────────────────────────────────

impl<C: Currency> Default for CalculationAmount<C> {
    fn default() -> Self {
        Self::ZERO
    }
}

// ─── Ordering ───────────────────────────────────────────────────────

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
        assert!(CalcUsd::ZERO.is_zero());
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
    fn sum_iterator() {
        let amounts = [
            CalcUsd::from_major(dec!(1.10)),
            CalcUsd::from_major(dec!(2.20)),
            CalcUsd::from_major(dec!(3.30)),
        ];
        let total: CalcUsd = amounts.into_iter().sum();
        assert_eq!(total.to_major(), dec!(6.60));
    }

    #[test]
    fn sum_reference_iterator() {
        let amounts = [
            CalcUsd::from_major(dec!(1.10)),
            CalcUsd::from_major(dec!(2.20)),
            CalcUsd::from_major(dec!(3.30)),
        ];
        let total: CalcUsd = amounts.iter().sum();
        assert_eq!(total.to_major(), dec!(6.60));
    }

    #[test]
    fn is_negative() {
        assert!(CalcUsd::from_major(dec!(-1.00)).is_negative());
        assert!(!CalcUsd::from_major(dec!(1.00)).is_negative());
        assert!(!CalcUsd::ZERO.is_negative());
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
    fn default_is_zero() {
        let d: CalcUsd = Default::default();
        assert!(d.is_zero());
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
    fn from_minor_units_lossless() {
        let cents = MinorUnits::<Usd>::from(12345u64);
        let calc = CalculationAmount::from(cents);
        assert_eq!(calc.to_major(), dec!(123.45));
    }

    #[test]
    fn from_minor_units_via_to_calc() {
        let cents = MinorUnits::<Usd>::from(12345u64);
        let calc = cents.to_calc();
        assert_eq!(calc.to_major(), dec!(123.45));
    }

    #[test]
    fn round_up_rounds_away_from_zero() {
        let calc = CalcUsd::from_major(dec!(3.287671));
        let rounded = calc.round_up();
        assert_eq!(rounded.into_inner(), 329);
    }

    #[test]
    fn round_down_rounds_toward_zero() {
        let calc = CalcUsd::from_major(dec!(3.287671));
        let rounded = calc.round_down();
        assert_eq!(rounded.into_inner(), 328);
    }

    #[test]
    fn round_bankers_at_midpoint() {
        let calc = CalcUsd::from_major(dec!(3.285));
        let rounded = calc.round_bankers();
        assert_eq!(rounded.into_inner(), 328);

        let calc2 = CalcUsd::from_major(dec!(3.295));
        let rounded2 = calc2.round_bankers();
        assert_eq!(rounded2.into_inner(), 330);
    }

    #[test]
    fn round_up_btc() {
        let calc = CalcBtc::from_major(dec!(0.001666666666));
        let rounded = calc.round_up();
        assert_eq!(rounded.into_inner(), 166667);
    }

    #[test]
    fn round_trip_lossless() {
        let original = MinorUnits::<Usd>::from(329u64);
        let calc = original.to_calc();
        let back = calc.round_up();
        assert_eq!(original, back);
    }

    #[test]
    fn round_zero() {
        let calc = CalcUsd::ZERO;
        assert_eq!(calc.round_up().into_inner(), 0);
        assert_eq!(calc.round_down().into_inner(), 0);
    }

    #[test]
    fn round_to_signed() {
        let calc = CalcUsd::from_major(dec!(-3.287671));
        let signed = calc.round_to_signed(RoundingStrategy::AwayFromZero);
        assert_eq!(signed.into_inner(), -329);
    }

    #[test]
    fn interest_precision_improvement() {
        let principal = CalcUsd::from_major(dec!(10000));
        let rate = dec!(0.12);
        let days_in_year = dec!(365);

        let total_unrounded: CalcUsd = (0..30).map(|_| principal * rate / days_in_year).sum();
        let sum_then_round = total_unrounded.round_up();

        let round_then_sum: MinorUnits<Usd> = (0..30)
            .map(|_| (principal * rate / days_in_year).round_up())
            .sum();

        assert!(sum_then_round <= round_then_sum);
    }
}
