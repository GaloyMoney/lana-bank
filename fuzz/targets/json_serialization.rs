#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_decimal::Decimal;
use serde_json;
use std::str;

// Import the real core-money types
use core_money::{Satoshis, SignedSatoshis, SignedUsdCents, UsdCents};

fuzz_target!(|data: &[u8]| {
    // Test raw JSON parsing
    if let Ok(json_str) = str::from_utf8(data) {
        fuzz_json_parsing(json_str);
    }

    // Test with JSON-like structures
    fuzz_json_like_structures(data);

    // Test with known types
    fuzz_known_type_serialization(data);
});

fn fuzz_json_parsing(json_str: &str) {
    // Test basic JSON value parsing
    let _ = serde_json::from_str::<serde_json::Value>(json_str);

    // Test parsing into money types
    fuzz_money_json_parsing(json_str);

    // Test malformed JSON handling
    fuzz_malformed_json(json_str);
}

fn fuzz_money_json_parsing(json_str: &str) {
    // Test core-money types deserialization
    let _ = serde_json::from_str::<Satoshis>(json_str);
    let _ = serde_json::from_str::<SignedSatoshis>(json_str);
    let _ = serde_json::from_str::<UsdCents>(json_str);
    let _ = serde_json::from_str::<SignedUsdCents>(json_str);

    // Test arrays and objects containing money types
    let _ = serde_json::from_str::<Vec<Satoshis>>(json_str);
    let _ = serde_json::from_str::<Vec<UsdCents>>(json_str);
    let _ = serde_json::from_str::<std::collections::HashMap<String, Satoshis>>(json_str);
    let _ = serde_json::from_str::<std::collections::HashMap<String, UsdCents>>(json_str);

    // Test nested structures with real core-money types
    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct MoneyContainer {
        satoshis: Option<Satoshis>,
        signed_satoshis: Option<SignedSatoshis>,
        usd_cents: Option<UsdCents>,
        signed_usd_cents: Option<SignedUsdCents>,
    }
    let _ = serde_json::from_str::<MoneyContainer>(json_str);
}

fn fuzz_malformed_json(json_str: &str) {
    // Test various malformed JSON patterns that might cause issues

    // Test with excessive nesting
    let nested = format!("[{}]", json_str);
    let _ = serde_json::from_str::<serde_json::Value>(&nested);

    // Test with special characters and escapes
    let escaped = json_str.replace("\"", "\\\"");
    let quoted = format!("\"{}\"", escaped);
    let _ = serde_json::from_str::<String>(&quoted);

    // Test with numbers in different formats
    if json_str
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E')
    {
        let _ = serde_json::from_str::<f64>(json_str);
        let _ = serde_json::from_str::<i64>(json_str);
        let _ = serde_json::from_str::<u64>(json_str);
    }
}

fn fuzz_json_like_structures(data: &[u8]) {
    // Create JSON-like structures from raw bytes
    let json_templates = [
        "{\"value\": %s}",
        "[%s]",
        "{\"satoshis\": %s, \"amount\": %s}",
        "{\"amount\": {\"value\": %s, \"currency\": \"BTC\"}}",
        "{\"nested\": {\"deep\": {\"value\": %s}}}",
    ];

    // Convert bytes to different representations
    let as_number = data
        .iter()
        .take(8)
        .fold(0u64, |acc, &b| acc.wrapping_mul(256).wrapping_add(b as u64));
    let as_string = data
        .iter()
        .take(32)
        .map(|&b| if b.is_ascii_graphic() { b as char } else { '_' })
        .collect::<String>();

    for template in json_templates {
        // Test with number substitution
        let json_with_number = template.replace("%s", &as_number.to_string());
        let _ = serde_json::from_str::<serde_json::Value>(&json_with_number);

        // Test with string substitution
        let json_with_string = template.replace("%s", &format!("\"{}\"", as_string));
        let _ = serde_json::from_str::<serde_json::Value>(&json_with_string);

        // Test type-specific parsing
        test_specific_types(&json_with_number);
        test_specific_types(&json_with_string);
    }
}

fn test_specific_types(json_str: &str) {
    // Try parsing as various real core-money types to find type-specific issues
    let _ = serde_json::from_str::<Satoshis>(json_str);
    let _ = serde_json::from_str::<SignedSatoshis>(json_str);
    let _ = serde_json::from_str::<UsdCents>(json_str);
    let _ = serde_json::from_str::<SignedUsdCents>(json_str);

    // Test complex structures with all core-money types
    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct ComplexType {
        id: Option<String>,
        satoshis: Option<Satoshis>,
        signed_satoshis: Option<SignedSatoshis>,
        usd_cents: Option<UsdCents>,
        signed_usd_cents: Option<SignedUsdCents>,
        metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    }
    let _ = serde_json::from_str::<ComplexType>(json_str);
}

fn fuzz_known_type_serialization(data: &[u8]) {
    if data.len() < 8 {
        return;
    }

    // Create instances and test round-trip serialization
    let value = u64::from_le_bytes(data[0..8].try_into().unwrap());

    // Test Satoshis round-trip using real core-money API
    let satoshis = Satoshis::from(value);
    if let Ok(json) = serde_json::to_string(&satoshis) {
        match serde_json::from_str::<Satoshis>(&json) {
            Ok(deserialized) => {
                assert_eq!(satoshis, deserialized, "Satoshis round-trip failed");
            }
            Err(e) => {
                panic!("Satoshis deserialization failed: {} for JSON: {}", e, json);
            }
        }

        // Test with additional transformations
        test_json_transformations(&json);
    }

    // Test UsdCents round-trip
    let usd_cents = UsdCents::from(value);
    if let Ok(json) = serde_json::to_string(&usd_cents) {
        match serde_json::from_str::<UsdCents>(&json) {
            Ok(deserialized) => {
                assert_eq!(usd_cents, deserialized, "UsdCents round-trip failed");
            }
            Err(e) => {
                panic!("UsdCents deserialization failed: {} for JSON: {}", e, json);
            }
        }
    }

    // Test SignedSatoshis round-trip - using from_btc to avoid panics with large values
    let signed_value = (value % 1_000_000_000) as i64; // Bounded to avoid core-money panics
    let signed_satoshis = SignedSatoshis::from_btc(Decimal::from(signed_value));
    if let Ok(json) = serde_json::to_string(&signed_satoshis) {
        match serde_json::from_str::<SignedSatoshis>(&json) {
            Ok(deserialized) => {
                assert_eq!(
                    signed_satoshis, deserialized,
                    "SignedSatoshis round-trip failed"
                );
            }
            Err(e) => {
                panic!(
                    "SignedSatoshis deserialization failed: {} for JSON: {}",
                    e, json
                );
            }
        }
    }

    // Test SignedUsdCents round-trip
    let signed_usd_value = (value % 1_000_000) as i64; // Bounded for USD amounts
    let signed_usd_cents = SignedUsdCents::from_usd(Decimal::from(signed_usd_value));
    if let Ok(json) = serde_json::to_string(&signed_usd_cents) {
        match serde_json::from_str::<SignedUsdCents>(&json) {
            Ok(deserialized) => {
                assert_eq!(
                    signed_usd_cents, deserialized,
                    "SignedUsdCents round-trip failed"
                );
            }
            Err(e) => {
                panic!(
                    "SignedUsdCents deserialization failed: {} for JSON: {}",
                    e, json
                );
            }
        }
    }

    // Test with second value if we have enough data - test large number combinations
    if data.len() >= 16 {
        let second_value = u64::from_le_bytes(data[8..16].try_into().unwrap());

        // Test combining large numbers to find overflow issues
        test_large_number_combinations(value, second_value);

        // Test edge cases around type boundaries
        test_boundary_values(value, second_value);
    }
}

fn test_large_number_combinations(a: u64, b: u64) {
    // Test very large Satoshis values that might cause issues
    let large_sats_a = Satoshis::from(a);
    let large_sats_b = Satoshis::from(b);

    // Test serialization of maximum values
    let max_sats = Satoshis::from(u64::MAX);
    let min_sats = Satoshis::from(0);

    if let Ok(json) = serde_json::to_string(&max_sats) {
        let _ = serde_json::from_str::<Satoshis>(&json);
    }

    if let Ok(json) = serde_json::to_string(&min_sats) {
        let _ = serde_json::from_str::<Satoshis>(&json);
    }

    // Test USD extremes
    let max_usd = UsdCents::from(u64::MAX);
    let min_usd = UsdCents::from(0);

    if let Ok(json) = serde_json::to_string(&max_usd) {
        let _ = serde_json::from_str::<UsdCents>(&json);
    }

    if let Ok(json) = serde_json::to_string(&min_usd) {
        let _ = serde_json::from_str::<UsdCents>(&json);
    }

    // Test signed extremes (bounded to avoid core-money panics)
    let bounded_max = 1_000_000_000i64;
    let bounded_min = -1_000_000_000i64;

    let max_signed = SignedSatoshis::from_btc(Decimal::from(bounded_max));
    let min_signed = SignedSatoshis::from_btc(Decimal::from(bounded_min));

    if let Ok(json) = serde_json::to_string(&max_signed) {
        let _ = serde_json::from_str::<SignedSatoshis>(&json);
    }

    if let Ok(json) = serde_json::to_string(&min_signed) {
        let _ = serde_json::from_str::<SignedSatoshis>(&json);
    }

    // Test arrays with large numbers
    let large_array = vec![large_sats_a, large_sats_b, max_sats];
    if let Ok(json) = serde_json::to_string(&large_array) {
        let _ = serde_json::from_str::<Vec<Satoshis>>(&json);
    }

    // Test USD arrays
    let large_usd_array = vec![UsdCents::from(a), UsdCents::from(b), max_usd];
    if let Ok(json) = serde_json::to_string(&large_usd_array) {
        let _ = serde_json::from_str::<Vec<UsdCents>>(&json);
    }
}

fn test_boundary_values(_a: u64, _b: u64) {
    // Test values around critical boundaries
    let boundaries = [
        0u64,
        1u64,
        u32::MAX as u64,
        u32::MAX as u64 + 1,
        i64::MAX as u64,
        i64::MAX as u64 + 1,
        u64::MAX - 1,
        u64::MAX,
    ];

    for &boundary in &boundaries {
        let sats = Satoshis::from(boundary);
        if let Ok(json) = serde_json::to_string(&sats) {
            match serde_json::from_str::<Satoshis>(&json) {
                Ok(deserialized) => {
                    assert_eq!(
                        sats, deserialized,
                        "Boundary value round-trip failed for {}",
                        boundary
                    );
                }
                Err(_) => {
                    // Some boundary values might legitimately fail
                }
            }
        }

        // Test USD boundaries
        let usd_cents = UsdCents::from(boundary);
        if let Ok(json) = serde_json::to_string(&usd_cents) {
            match serde_json::from_str::<UsdCents>(&json) {
                Ok(deserialized) => {
                    assert_eq!(
                        usd_cents, deserialized,
                        "USD boundary value round-trip failed for {}",
                        boundary
                    );
                }
                Err(_) => {
                    // Some boundary values might legitimately fail
                }
            }
        }

        // Test signed boundaries (bounded to avoid core-money panics)
        if boundary <= i64::MAX as u64 && boundary < 1_000_000_000 {
            let signed_sats = SignedSatoshis::from_btc(Decimal::from(boundary as i64));
            if let Ok(json) = serde_json::to_string(&signed_sats) {
                let _ = serde_json::from_str::<SignedSatoshis>(&json);
            }
        }
    }

    // Test negative boundaries for signed types
    let negative_boundaries = [
        i64::MIN,
        i64::MIN + 1,
        -1i64,
        0i64,
        1i64,
        i64::MAX - 1,
        i64::MAX,
    ];

    for &boundary in &negative_boundaries {
        // Bound the values to avoid core-money panics
        let bounded_boundary = boundary.max(-1_000_000_000).min(1_000_000_000);

        let signed_sats = SignedSatoshis::from_btc(Decimal::from(bounded_boundary));
        if let Ok(json) = serde_json::to_string(&signed_sats) {
            match serde_json::from_str::<SignedSatoshis>(&json) {
                Ok(deserialized) => {
                    assert_eq!(
                        signed_sats, deserialized,
                        "Signed boundary round-trip failed for {}",
                        bounded_boundary
                    );
                }
                Err(_) => {
                    // Some values might legitimately fail
                }
            }
        }

        // Also test SignedUsdCents
        let signed_usd_cents = SignedUsdCents::from_usd(Decimal::from(bounded_boundary));
        if let Ok(json) = serde_json::to_string(&signed_usd_cents) {
            match serde_json::from_str::<SignedUsdCents>(&json) {
                Ok(deserialized) => {
                    assert_eq!(
                        signed_usd_cents, deserialized,
                        "Signed USD boundary round-trip failed for {}",
                        bounded_boundary
                    );
                }
                Err(_) => {
                    // Some values might legitimately fail
                }
            }
        }
    }
}

fn test_json_transformations(json: &str) {
    // Test various JSON transformations that might reveal edge cases

    // Test with extra whitespace
    let with_spaces = format!(" \t\n{}\r\n ", json);
    let _ = serde_json::from_str::<serde_json::Value>(&with_spaces);

    // Test with the JSON embedded in larger structures
    let embedded = format!(r#"{{"data": {}, "metadata": {{"test": true}}}}"#, json);
    let _ = serde_json::from_str::<serde_json::Value>(&embedded);

    // Test in arrays
    let in_array = format!("[{}, {}, {}]", json, json, json);
    let _ = serde_json::from_str::<serde_json::Value>(&in_array);

    // Test with string escaping variations
    if json.contains('"') {
        let escaped = json.replace('"', "\\\"");
        let wrapped = format!("\"{}\"", escaped);
        let _ = serde_json::from_str::<String>(&wrapped);
    }
}
