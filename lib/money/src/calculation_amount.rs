use rust_decimal::{Decimal, RoundingStrategy, prelude::ToPrimitive};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, Mul};

use crate::{Currency, MinorUnits, Precision};

/// High-precision monetary amount for multi-step financial calculations.
///
/// Wraps `Decimal` (28 significant digits) with currency type safety and a
/// stored `Precision` for regulatory rounding. Arithmetic stays at full ~28dp
/// precision; rounding is applied only at explicit rounding boundaries
/// (`round`, `round_to_minor_units`).
///
/// Use this type only when you need multi-step accumulation before rounding
/// (e.g., interest accrual). For single-expression arithmetic that immediately
/// rounds, use `MinorUnits::from_major_rounded` instead.
///
/// Intentionally does NOT implement `Serialize`, `Deserialize`, `sqlx::Encode`,
/// `sqlx::Decode`, or GraphQL scalars — the only way to persist is through
/// explicit rounding.
#[derive(Clone, Copy)]
pub struct CalculationAmount<C: Currency> {
    value: Decimal,
    precision: Precision,
    _currency: PhantomData<C>,
}

// ─── Construction ────────────────────────────────────────────────────

impl<C: Currency> CalculationAmount<C> {
    pub fn from_major(major: Decimal, precision: Precision) -> Self {
        Self {
            value: major,
            precision,
            _currency: PhantomData,
        }
    }

    pub fn from_minor(units: MinorUnits<C>, precision: Precision) -> Self {
        Self::from_major(units.to_major(), precision)
    }
}

// ─── The Rounding Boundary ──────────────────────────────────────────

impl<C: Currency> CalculationAmount<C> {
    /// Round to the stored precision with the given strategy.
    /// For regulatory/reporting storage.
    pub fn round(&self, strategy: RoundingStrategy) -> Decimal {
        self.value
            .round_dp_with_strategy(self.precision.dp(), strategy)
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
            self.value >= Decimal::ZERO,
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
}

// ─── Same-Currency Arithmetic (full precision, no auto-rounding) ────

impl<C: Currency> Add for CalculationAmount<C> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        debug_assert_eq!(self.precision, rhs.precision);
        Self {
            value: self.value + rhs.value,
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
    use rust_decimal::RoundingStrategy;
    use rust_decimal_macros::dec;

    use super::*;
    use crate::{Btc, Usd};

    type CalcUsd = CalculationAmount<Usd>;
    type CalcBtc = CalculationAmount<Btc>;

    fn p6() -> Precision {
        Precision::try_new(6).unwrap()
    }

    #[test]
    fn from_major_preserves_value() {
        let calc = CalcUsd::from_major(dec!(123.456789), p6());
        // 123.456789 major = 12345.6789 minor → rounds to 12346 cents (AwayFromZero)
        let rounded = calc.round_to_minor_units(RoundingStrategy::AwayFromZero);
        assert_eq!(rounded.into_inner(), 12346);
        // Verify via precision rounding at high dp
        assert_eq!(
            calc.round(RoundingStrategy::MidpointAwayFromZero),
            dec!(123.456789)
        );
    }

    #[test]
    fn add_same_currency() {
        let a = CalcUsd::from_major(dec!(1.50), p6());
        let b = CalcUsd::from_major(dec!(2.75), p6());
        // 1.50 + 2.75 = 4.25 → 425 cents
        assert_eq!(
            (a + b)
                .round_to_minor_units(RoundingStrategy::AwayFromZero)
                .into_inner(),
            425
        );
    }

    #[test]
    fn mul_by_decimal() {
        let amt = CalcUsd::from_major(dec!(100.00), p6());
        let rate = dec!(0.12);
        // 100 * 0.12 = 12.00 → 1200 cents
        assert_eq!(
            (amt * rate)
                .round_to_minor_units(RoundingStrategy::AwayFromZero)
                .into_inner(),
            1200
        );
    }

    #[test]
    fn fold_summation() {
        let amounts = [
            CalcUsd::from_major(dec!(1.10), p6()),
            CalcUsd::from_major(dec!(2.20), p6()),
            CalcUsd::from_major(dec!(3.30), p6()),
        ];
        let total = amounts
            .into_iter()
            .fold(CalcUsd::from_major(dec!(0), p6()), |acc, x| acc + x);
        // 1.10 + 2.20 + 3.30 = 6.60 → 660 cents
        assert_eq!(
            total
                .round_to_minor_units(RoundingStrategy::AwayFromZero)
                .into_inner(),
            660
        );
    }

    #[test]
    fn debug_format() {
        let a = CalcUsd::from_major(dec!(1.23), p6());
        assert_eq!(format!("{:?}", a), "Calc<USD>(1.23)");
    }

    #[test]
    fn display_format() {
        let a = CalcUsd::from_major(dec!(1.23), p6());
        assert_eq!(format!("{}", a), "~1.23 USD");
    }

    #[test]
    fn ordering() {
        let a = CalcUsd::from_major(dec!(1.00), p6());
        let b = CalcUsd::from_major(dec!(2.00), p6());
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, CalcUsd::from_major(dec!(1.00), p6()));
    }

    #[test]
    fn from_minor_units() {
        let cents = MinorUnits::<Usd>::from(12345u64);
        let calc = CalculationAmount::from_minor(cents, p6());
        // Round-trip: 12345 cents → from_minor → round back → 12345
        let back = calc.round_to_minor_units(RoundingStrategy::AwayFromZero);
        assert_eq!(back.into_inner(), 12345);
    }

    #[test]
    fn round_to_minor_units_away_from_zero() {
        let calc = CalcUsd::from_major(dec!(3.287671), p6());
        let rounded = calc.round_to_minor_units(RoundingStrategy::AwayFromZero);
        assert_eq!(rounded.into_inner(), 329);
    }

    #[test]
    fn round_to_minor_units_to_zero() {
        let calc = CalcUsd::from_major(dec!(3.287671), p6());
        let rounded = calc.round_to_minor_units(RoundingStrategy::ToZero);
        assert_eq!(rounded.into_inner(), 328);
    }

    #[test]
    fn round_to_minor_units_btc() {
        let calc = CalcBtc::from_major(dec!(0.001666666666), p6());
        let rounded = calc.round_to_minor_units(RoundingStrategy::AwayFromZero);
        assert_eq!(rounded.into_inner(), 166667);
    }

    #[test]
    fn round_trip_lossless() {
        let original = MinorUnits::<Usd>::from(329u64);
        let calc = CalculationAmount::from_minor(original, p6());
        let back = calc.round_to_minor_units(RoundingStrategy::AwayFromZero);
        assert_eq!(original, back);
    }

    #[test]
    fn round_zero() {
        let calc = CalcUsd::from_major(dec!(0), p6());
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
        let strategy = RoundingStrategy::MidpointAwayFromZero;

        let calc = CalcUsd::from_major(dec!(3.287671), p6());
        assert_eq!(calc.round(strategy), dec!(3.287671));

        let calc2 = CalcUsd::from_major(dec!(3.2876714999), p6());
        assert_eq!(calc2.round(strategy), dec!(3.287671));

        let calc3 = CalcUsd::from_major(dec!(3.2876715), p6());
        assert_eq!(calc3.round(strategy), dec!(3.287672));
    }

    #[test]
    fn arithmetic_preserves_full_precision() {
        let principal = CalcUsd::from_major(dec!(10000), p6());
        // Mirror how interest_for_period works: principal * (rate * days / year)
        let daily_factor = dec!(0.12) / dec!(365);

        let daily = principal * daily_factor;

        // Sum 30 days — should maintain full precision
        let total = (0..30).fold(CalcUsd::from_major(dec!(0), p6()), |acc, _| acc + daily);
        let sum_then_round = total.round_to_minor_units(RoundingStrategy::AwayFromZero);

        // Round each day then sum (the wrong way)
        let round_then_sum: MinorUnits<Usd> = (0..30)
            .map(|_| daily.round_to_minor_units(RoundingStrategy::AwayFromZero))
            .sum();

        // sum-then-round should be <= round-then-sum (no precision loss)
        assert!(sum_then_round <= round_then_sum);
    }
}
