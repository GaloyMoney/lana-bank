#![no_main]

use core_accounting::primitives::{AccountCode, AccountCodeSection, AccountIdOrCode, AccountName};
use libfuzzer_sys::fuzz_target;
use std::str;

fuzz_target!(|data: &[u8]| {
    // Test string parsing with raw bytes converted to UTF-8
    if let Ok(input_str) = str::from_utf8(data) {
        fuzz_account_parsing(input_str);
    }

    // Test with truncated strings of various lengths
    for len in 1..=data.len().min(256) {
        if let Ok(truncated) = str::from_utf8(&data[..len]) {
            fuzz_account_parsing(truncated);
        }
    }

    // Test with replaced characters to find edge cases
    if data.len() > 0 {
        let mut modified = data.to_vec();

        // Replace random bytes with special characters that might cause issues
        let special_chars = [
            b'-', b'_', b'.', b':', b'/', b'\\', b' ', b'\t', b'\n', b'\r',
        ];
        for (i, &special) in special_chars.iter().enumerate() {
            if i < modified.len() {
                modified[i] = special;
                if let Ok(modified_str) = str::from_utf8(&modified) {
                    fuzz_account_parsing(modified_str);
                }
            }
        }
    }
});

fn fuzz_account_parsing(input: &str) {
    // Test AccountName parsing
    fuzz_account_name(input);

    // Test AccountCode parsing
    fuzz_account_code(input);

    // Test AccountCodeSection parsing
    fuzz_account_code_section(input);

    // Test AccountIdOrCode parsing
    fuzz_account_id_or_code(input);

    // Test with common prefixes and suffixes
    let prefixes = [
        "",
        "assets:",
        "liabilities:",
        "equity:",
        "revenue:",
        "expenses:",
    ];
    let suffixes = ["", ":main", ":sub", "_001", "-temp"];

    for prefix in prefixes {
        for suffix in suffixes {
            let combined = format!("{}{}{}", prefix, input, suffix);
            test_all_parsers(&combined);
        }
    }

    // Test with numeric patterns that might be UUIDs or codes
    if input.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        test_all_parsers(input);
    }
}

fn fuzz_account_name(input: &str) {
    match input.parse::<AccountName>() {
        Ok(account_name) => {
            // If parsing succeeded, verify round-trip
            let as_string = account_name.to_string();

            // Should be able to parse it again
            match as_string.parse::<AccountName>() {
                Ok(reparsed) => {
                    assert_eq!(account_name, reparsed, "AccountName round-trip failed");
                }
                Err(_) => {
                    panic!("AccountName round-trip parsing failed for: {}", as_string);
                }
            }

            // Test serialization if serde is available
            test_serde_round_trip_account_name(&account_name);
        }
        Err(_) => {
            // Parsing failed - this is expected for invalid input
            // Just verify it doesn't crash
        }
    }
}

fn fuzz_account_code(input: &str) {
    match input.parse::<AccountCode>() {
        Ok(account_code) => {
            // Test round-trip conversion
            let as_string = account_code.to_string();
            match as_string.parse::<AccountCode>() {
                Ok(reparsed) => {
                    assert_eq!(account_code, reparsed, "AccountCode round-trip failed");
                }
                Err(_) => {
                    panic!("AccountCode round-trip parsing failed for: {}", as_string);
                }
            }

            // Test section extraction if available
            let _ = account_code.section();

            test_serde_round_trip_account_code(&account_code);
        }
        Err(_) => {
            // Expected for invalid input
        }
    }
}

fn fuzz_account_code_section(input: &str) {
    match input.parse::<AccountCodeSection>() {
        Ok(section) => {
            // Test round-trip
            let as_string = section.to_string();
            match as_string.parse::<AccountCodeSection>() {
                Ok(reparsed) => {
                    assert_eq!(section, reparsed, "AccountCodeSection round-trip failed");
                }
                Err(_) => {
                    panic!(
                        "AccountCodeSection round-trip parsing failed for: {}",
                        as_string
                    );
                }
            }

            test_serde_round_trip_account_code_section(&section);
        }
        Err(_) => {
            // Expected for invalid input
        }
    }
}

fn fuzz_account_id_or_code(input: &str) {
    match input.parse::<AccountIdOrCode>() {
        Ok(id_or_code) => {
            // Test round-trip
            let as_string = id_or_code.to_string();
            match as_string.parse::<AccountIdOrCode>() {
                Ok(reparsed) => {
                    assert_eq!(id_or_code, reparsed, "AccountIdOrCode round-trip failed");
                }
                Err(_) => {
                    panic!(
                        "AccountIdOrCode round-trip parsing failed for: {}",
                        as_string
                    );
                }
            }

            test_serde_round_trip_account_id_or_code(&id_or_code);
        }
        Err(_) => {
            // Expected for invalid input
        }
    }
}

fn test_all_parsers(input: &str) {
    // Test all parser types with the same input to find inconsistencies
    let _ = input.parse::<AccountName>();
    let _ = input.parse::<AccountCode>();
    let _ = input.parse::<AccountCodeSection>();
    let _ = input.parse::<AccountIdOrCode>();
}

// Serde round-trip tests to ensure serialization/deserialization is robust
fn test_serde_round_trip_account_name(account_name: &AccountName) {
    // Test JSON serialization round-trip
    if let Ok(json) = serde_json::to_string(account_name) {
        match serde_json::from_str::<AccountName>(&json) {
            Ok(deserialized) => {
                assert_eq!(
                    account_name, &deserialized,
                    "AccountName JSON round-trip failed"
                );
            }
            Err(_) => {
                panic!("AccountName JSON deserialization failed for: {}", json);
            }
        }
    }
}

fn test_serde_round_trip_account_code(account_code: &AccountCode) {
    if let Ok(json) = serde_json::to_string(account_code) {
        match serde_json::from_str::<AccountCode>(&json) {
            Ok(deserialized) => {
                assert_eq!(
                    account_code, &deserialized,
                    "AccountCode JSON round-trip failed"
                );
            }
            Err(_) => {
                panic!("AccountCode JSON deserialization failed for: {}", json);
            }
        }
    }
}

fn test_serde_round_trip_account_code_section(section: &AccountCodeSection) {
    if let Ok(json) = serde_json::to_string(section) {
        match serde_json::from_str::<AccountCodeSection>(&json) {
            Ok(deserialized) => {
                assert_eq!(
                    section, &deserialized,
                    "AccountCodeSection JSON round-trip failed"
                );
            }
            Err(_) => {
                panic!(
                    "AccountCodeSection JSON deserialization failed for: {}",
                    json
                );
            }
        }
    }
}

fn test_serde_round_trip_account_id_or_code(id_or_code: &AccountIdOrCode) {
    if let Ok(json) = serde_json::to_string(id_or_code) {
        match serde_json::from_str::<AccountIdOrCode>(&json) {
            Ok(deserialized) => {
                assert_eq!(
                    id_or_code, &deserialized,
                    "AccountIdOrCode JSON round-trip failed"
                );
            }
            Err(_) => {
                panic!("AccountIdOrCode JSON deserialization failed for: {}", json);
            }
        }
    }
}
