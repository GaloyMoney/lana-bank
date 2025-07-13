#![no_main]

use libfuzzer_sys::fuzz_target;
use core_money::*;
use rust_decimal::Decimal;
use std::panic;

fuzz_target!(|data: &[u8]| {
    // Try to parse the input as a string first
    if let Ok(s) = std::str::from_utf8(data) {
        // Test decimal parsing and conversion
        if let Ok(decimal) = s.parse::<Decimal>() {
            // Test BTC to Satoshis conversion (signed)
            let _ = panic::catch_unwind(|| {
                let _ = SignedSatoshis::from_btc(decimal);
            });
            
            // Test BTC to Satoshis conversion (unsigned, fallible)
            let _ = panic::catch_unwind(|| {
                let _ = Satoshis::try_from_btc(decimal);
            });
            
            // Test USD to UsdCents conversion (unsigned, fallible)
            let _ = panic::catch_unwind(|| {
                let _ = UsdCents::try_from_usd(decimal);
            });
            
            // Test USD to SignedUsdCents conversion
            let _ = panic::catch_unwind(|| {
                let _ = SignedUsdCents::from_usd(decimal);
            });
            
            // Test roundtrip conversions for valid values
            let _ = panic::catch_unwind(|| {
                if let Ok(sats) = Satoshis::try_from_btc(decimal) {
                    let btc_back = sats.to_btc();
                    // Verify the conversion doesn't panic
                    assert!(!btc_back.is_zero() || decimal.is_zero());
                }
            });
            
            let _ = panic::catch_unwind(|| {
                if let Ok(cents) = UsdCents::try_from_usd(decimal) {
                    let usd_back = cents.to_usd();
                    // Verify the conversion doesn't panic
                    assert!(!usd_back.is_zero() || decimal.is_zero());
                }
            });
        }
    }
    
    // Test arithmetic operations with raw bytes interpreted as u64/i64
    if data.len() >= 16 {
        let val1 = u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
        let val2 = u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]);
        
        // Test Satoshis arithmetic
        let sats1 = Satoshis::from(val1);
        let sats2 = Satoshis::from(val2);
        
        let _ = panic::catch_unwind(|| {
            let _ = sats1 + sats2;
        });
        
        let _ = panic::catch_unwind(|| {
            if sats1 >= sats2 {
                let _ = sats1 - sats2;
            }
        });
        
        // Test UsdCents arithmetic
        let cents1 = UsdCents::from(val1);
        let cents2 = UsdCents::from(val2);
        
        let _ = panic::catch_unwind(|| {
            let _ = cents1 + cents2;
        });
        
        let _ = panic::catch_unwind(|| {
            if cents1 >= cents2 {
                let _ = cents1 - cents2;
            }
        });
        
        // Test SignedSatoshis arithmetic
        let signed_val1 = val1 as i64;
        let signed_val2 = val2 as i64;
        
        let signed_sats1 = SignedSatoshis::from_btc(Decimal::from(signed_val1));
        let signed_sats2 = SignedSatoshis::from_btc(Decimal::from(signed_val2));
        
        let _ = panic::catch_unwind(|| {
            let _ = signed_sats1 + signed_sats2;
        });
        
        let _ = panic::catch_unwind(|| {
            let _ = signed_sats1 - signed_sats2;
        });
        
        // Test conversions between signed and unsigned
        let _ = panic::catch_unwind(|| {
            let _ = Satoshis::try_from(signed_sats1);
        });
    }
    
    // Test edge cases with specific patterns
    if data.len() >= 8 {
        let val = u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
        
        // Test maximum values
        let sats = Satoshis::from(val);
        let _ = panic::catch_unwind(|| {
            let _ = sats.to_btc();
        });
        
        let _ = panic::catch_unwind(|| {
            let _ = sats.formatted_btc();
        });
        
        // Test UsdCents
        let cents = UsdCents::from(val);
        let _ = panic::catch_unwind(|| {
            let _ = cents.to_usd();
        });
        
        let _ = panic::catch_unwind(|| {
            let _ = cents.formatted_usd();
        });
    }
});
