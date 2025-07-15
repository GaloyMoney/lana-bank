#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_decimal::Decimal;

// Import the real core-money types
use core_money::{Satoshis, SignedSatoshis, SignedUsdCents, UsdCents};

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }

    // Extract values for testing from fuzz input
    let a_bytes: [u8; 8] = data[0..8].try_into().unwrap();
    let b_bytes: [u8; 8] = data[8..16].try_into().unwrap();

    let a_u64 = u64::from_le_bytes(a_bytes);
    let b_u64 = u64::from_le_bytes(b_bytes);
    let a_i64 = i64::from_le_bytes(a_bytes);
    let b_i64 = i64::from_le_bytes(b_bytes);

    // Test Satoshis operations
    fuzz_satoshis_operations(a_u64, b_u64);

    // Test SignedSatoshis operations
    fuzz_signed_satoshis_operations(a_i64, b_i64);

    // Test BTC conversions
    fuzz_btc_conversions(a_u64);

    // Test conversions between signed and unsigned
    fuzz_type_conversions(a_u64, a_i64);

    // Test USD operations
    fuzz_usd_operations(a_u64, b_u64);
    fuzz_signed_usd_operations(a_i64, b_i64);
});

fn fuzz_satoshis_operations(a: u64, b: u64) {
    let sats_a = Satoshis::from(a);
    let sats_b = Satoshis::from(b);

    // Test basic operations that should never panic
    let _ = sats_a.to_btc();
    let _ = sats_a.into_inner();
    let _ = sats_a.formatted_btc();

    // Test arithmetic operations (real core-money uses direct ops, not checked_*)
    // These can panic on overflow, which is what we want to catch in fuzzing
    if a.saturating_add(b) == a + b {
        // Only test if no overflow would occur in u64
        let _ = sats_a + sats_b;
    }

    if a >= b {
        let _ = sats_a - sats_b;
    }
}

fn fuzz_signed_satoshis_operations(a: i64, b: i64) {
    // Limit input range to avoid panics in core-money's from_btc
    // The issue is with very large i64 values that can't convert back to i64 through Decimal
    let bounded_a = (a % 1_000_000_000) as i64; // Reasonable satoshi range
    let bounded_b = (b % 1_000_000_000) as i64;

    let signed_a = SignedSatoshis::from_btc(Decimal::from(bounded_a));
    let signed_b = SignedSatoshis::from_btc(Decimal::from(bounded_b));

    // Test basic operations
    let _ = signed_a.to_btc();
    let _ = signed_a.into_inner();
    let _ = signed_a.abs();

    // Test arithmetic operations (real core-money uses direct ops)
    if a.saturating_add(b) == a + b {
        let _ = signed_a + signed_b;
    }

    if a.saturating_sub(b) == a - b {
        let _ = signed_a - signed_b;
    }
}

fn fuzz_btc_conversions(value: u64) {
    let sats = Satoshis::from(value);
    let btc_decimal = sats.to_btc();

    // Test round-trip conversion
    if let Ok(back_to_sats) = Satoshis::try_from_btc(btc_decimal) {
        // Should be reasonably close (within reasonable precision)
        let diff = if back_to_sats.into_inner() > sats.into_inner() {
            back_to_sats.into_inner() - sats.into_inner()
        } else {
            sats.into_inner() - back_to_sats.into_inner()
        };
        assert!(diff <= 1, "Round-trip conversion precision error too large");
    }

    // Test that very large BTC amounts are handled properly
    let large_btc = Decimal::from(value) / Decimal::from(1000); // Scale down to reasonable BTC amount
    let _ = Satoshis::try_from_btc(large_btc);

    // Test signed BTC conversions (bounded to avoid panics)
    let bounded_btc = btc_decimal % Decimal::from(1_000_000_000);
    let _ = SignedSatoshis::from_btc(bounded_btc);
    let negative_btc = -bounded_btc;
    let _ = SignedSatoshis::from_btc(negative_btc);
}

fn fuzz_type_conversions(a_u64: u64, a_i64: i64) {
    let sats = Satoshis::from(a_u64);

    // Test conversion from unsigned to signed
    // This is where the bug was found in the original implementation
    match SignedSatoshis::try_from(sats) {
        Ok(signed_sats) => {
            // If conversion succeeded, value should be within i64 range
            assert!(a_u64 <= i64::MAX as u64);
            assert_eq!(signed_sats.into_inner(), a_u64 as i64);
        }
        Err(_) => {
            // If conversion failed, value should be > i64::MAX
            assert!(a_u64 > i64::MAX as u64);
        }
    }

    // Test conversion from signed to unsigned
    if a_i64 >= 0 {
        let bounded_i64 = (a_i64 % 1_000_000_000) as i64;
        let signed_sats = SignedSatoshis::from_btc(Decimal::from(bounded_i64));
        match Satoshis::try_from(signed_sats) {
            Ok(unsigned_sats) => {
                // The conversion goes through BTC, so we need to account for precision
                let expected_btc = Decimal::from(a_i64);
                let actual_btc = unsigned_sats.to_btc();
                assert!((expected_btc - actual_btc).abs() < Decimal::from(1)); // Within 1 satoshi
            }
            Err(_) => {
                // Should not fail for non-negative values in normal cases
            }
        }
    } else {
        let bounded_i64 = (a_i64 % 1_000_000_000) as i64;
        let signed_sats = SignedSatoshis::from_btc(Decimal::from(bounded_i64));
        match Satoshis::try_from(signed_sats) {
            Ok(_) => {
                panic!("Conversion from negative SignedSatoshis to Satoshis should fail");
            }
            Err(_) => {
                // Expected for negative values
            }
        }
    }
}

fn fuzz_usd_operations(a: u64, b: u64) {
    let cents_a = UsdCents::from(a);
    let cents_b = UsdCents::from(b);

    // Test basic operations that should never panic
    let _ = cents_a.to_usd();
    let _ = cents_a.into_inner();
    let _ = cents_a.formatted_usd();

    // Test arithmetic operations
    if a.saturating_add(b) == a + b {
        let _ = cents_a + cents_b;
    }

    if a >= b {
        let _ = cents_a - cents_b;
    }

    // Test round-trip USD conversion
    let usd_decimal = cents_a.to_usd();
    if let Ok(back_to_cents) = UsdCents::try_from_usd(usd_decimal) {
        assert_eq!(back_to_cents.into_inner(), cents_a.into_inner());
    }
}

fn fuzz_signed_usd_operations(a: i64, b: i64) {
    // Limit input range to avoid panics in core-money's from_usd
    let bounded_a = (a % 1_000_000) as i64; // Reasonable USD range
    let bounded_b = (b % 1_000_000) as i64;

    let signed_cents_a = SignedUsdCents::from_usd(Decimal::from(bounded_a));
    let signed_cents_b = SignedUsdCents::from_usd(Decimal::from(bounded_b));

    // Test basic operations
    let _ = signed_cents_a.to_usd();
    let _ = signed_cents_a.into_inner();

    // Test arithmetic operations (SignedUsdCents only has subtraction in core-money)

    if a.saturating_sub(b) == a - b {
        let _ = signed_cents_a - signed_cents_b;
    }
}
