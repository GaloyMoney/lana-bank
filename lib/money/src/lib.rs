#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod code;
mod currency;
mod error;
mod map;
mod units;

pub use code::*;
pub use currency::*;
pub use error::ConversionError;
pub use map::*;
pub use units::{MinorUnits, SignedMinorUnits};

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

pub type UsdCents = MinorUnits<Usd>;
pub type Satoshis = MinorUnits<Btc>;
pub type SignedUsdCents = SignedMinorUnits<Usd>;
pub type SignedSatoshis = SignedMinorUnits<Btc>;
pub type UntypedAmount = MinorUnits<Untyped>;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal::Decimal;

    #[test]
    fn static_currency_zero_and_one() {
        assert!(UsdCents::ZERO.is_zero());
        assert!(!UsdCents::ONE.is_zero());
        assert!(Satoshis::ZERO.is_zero());
    }

    #[test]
    fn static_currency_from_u64() {
        let cents = UsdCents::from(1500u64);
        assert_eq!(cents.into_inner(), 1500);
        assert_eq!(cents.currency(), CurrencyCode::USD);
    }

    #[test]
    fn to_major_works_for_static() {
        let cents = UsdCents::from(150u64);
        assert_eq!(cents.to_major(), Decimal::new(150, 2)); // $1.50

        let sats = Satoshis::from(100_000_000u64);
        assert_eq!(sats.to_major(), Decimal::from(1)); // 1 BTC
    }

    #[test]
    fn typed_to_untyped_conversion() {
        let cents = UsdCents::from(1500u64);
        let untyped: UntypedAmount = cents.into();

        assert_eq!(untyped.currency(), CurrencyCode::USD);
        assert_eq!(untyped.into_inner(), 1500);
        assert_eq!(untyped.to_major(), Decimal::new(1500, 2)); // $15.00
    }

    #[test]
    fn untyped_to_typed_downcast() {
        let cents = UsdCents::from(1500u64);
        let untyped: UntypedAmount = cents.into();

        // Correct downcast
        let back = untyped.to_typed::<Usd>().unwrap();
        assert_eq!(back, UsdCents::from(1500u64));

        // Wrong downcast
        assert!(untyped.to_typed::<Btc>().is_none());
    }

    #[test]
    fn untyped_try_from_major() {
        let amt = UntypedAmount::try_from_major(CurrencyCode::USD, Decimal::new(1500, 2)).unwrap();
        assert_eq!(amt.currency(), CurrencyCode::USD);
        assert_eq!(amt.into_inner(), 1500);

        let amt = UntypedAmount::try_from_major(CurrencyCode::BTC, Decimal::from(1)).unwrap();
        assert_eq!(amt.currency(), CurrencyCode::BTC);
        assert_eq!(amt.into_inner(), 100_000_000);
    }

    #[test]
    fn static_serde_is_bare_u64() {
        let cents = UsdCents::from(42u64);
        let json = serde_json::to_string(&cents).unwrap();
        assert_eq!(json, "42");

        let back: UsdCents = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cents);
    }

    #[test]
    fn untyped_serde_is_struct() {
        let cents = UsdCents::from(42u64);
        let untyped: UntypedAmount = cents.into();
        let json = serde_json::to_string(&untyped).unwrap();
        assert!(json.contains("\"currency\""));
        assert!(json.contains("\"minor_units\""));
        assert!(json.contains("\"minor_units_per_major\""));

        let back: UntypedAmount = serde_json::from_str(&json).unwrap();
        assert_eq!(back, untyped);
        assert_eq!(back.to_typed::<Usd>(), Some(UsdCents::from(42u64)));
    }

    #[test]
    fn arithmetic_works_for_static() {
        let a = UsdCents::from(100u64);
        let b = UsdCents::from(50u64);
        assert_eq!((a + b).into_inner(), 150);
        assert_eq!((a - b).into_inner(), 50);
    }

    // Arithmetic does NOT compile for Untyped — this is intentional.
    // let a: UntypedAmount = ...; let b: UntypedAmount = ...; a + b; // ERROR

    #[test]
    fn size_of_types() {
        // StaticCurrency: just u64 + ZSTs = 8 bytes
        assert_eq!(std::mem::size_of::<UsdCents>(), 8);
        assert_eq!(std::mem::size_of::<Satoshis>(), 8);

        // Untyped: u64 + CurrencyMeta (CurrencyCode + u64) + ZST
        // CurrencyCode is &'static str = pointer = 16 bytes (ptr + len)
        let untyped_size = std::mem::size_of::<UntypedAmount>();
        assert!(
            untyped_size > 8,
            "UntypedAmount should be larger: {untyped_size}"
        );
    }
}
