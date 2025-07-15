#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// Simplified versions of the money types for fuzz testing
const SATS_PER_BTC: Decimal = Decimal::from_parts(100_000_000, 0, 0, false, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Satoshis(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SignedSatoshis(i64);

#[derive(Debug)]
pub enum ConversionError {
    DecimalError,
    UnexpectedNegativeNumber(Decimal),
}

impl Satoshis {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn from(value: u64) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> u64 {
        self.0
    }

    pub fn to_btc(self) -> Decimal {
        Decimal::from(self.0) / SATS_PER_BTC
    }

    pub fn try_from_btc(btc: Decimal) -> Result<Self, ConversionError> {
        let sats = btc * SATS_PER_BTC;
        if sats < Decimal::ZERO {
            return Err(ConversionError::UnexpectedNegativeNumber(sats));
        }
        match u64::try_from(sats.trunc()) {
            Ok(val) => Ok(Self(val)),
            Err(_) => Err(ConversionError::DecimalError),
        }
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn formatted_btc(self) -> String {
        format!("{:.8}", self.to_btc())
    }
}

impl std::ops::Add<Satoshis> for Satoshis {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Satoshis(self.0 + other.0)
    }
}

impl std::ops::Sub<Satoshis> for Satoshis {
    type Output = Satoshis;

    fn sub(self, other: Satoshis) -> Satoshis {
        Satoshis(self.0 - other.0)
    }
}

impl std::ops::AddAssign for Satoshis {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl std::ops::SubAssign for Satoshis {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl SignedSatoshis {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn into_inner(self) -> i64 {
        self.0
    }

    pub fn to_btc(self) -> Decimal {
        Decimal::from(self.0) / SATS_PER_BTC
    }

    pub fn from_btc(btc: Decimal) -> Self {
        let sats = btc * SATS_PER_BTC;
        Self(i64::try_from(sats.trunc()).expect("Satoshis must be integer"))
    }

    pub fn abs(self) -> SignedSatoshis {
        SignedSatoshis(self.0.abs())
    }
}

impl From<Satoshis> for SignedSatoshis {
    fn from(sats: Satoshis) -> Self {
        Self(i64::try_from(sats.0).expect("Satoshis must be integer sized for i64"))
    }
}

impl std::ops::Add<SignedSatoshis> for SignedSatoshis {
    type Output = SignedSatoshis;

    fn add(self, other: SignedSatoshis) -> SignedSatoshis {
        SignedSatoshis(self.0 + other.0)
    }
}

impl std::ops::Sub<SignedSatoshis> for SignedSatoshis {
    type Output = SignedSatoshis;

    fn sub(self, other: SignedSatoshis) -> SignedSatoshis {
        SignedSatoshis(self.0 - other.0)
    }
}

impl TryFrom<SignedSatoshis> for Satoshis {
    type Error = ConversionError;

    fn try_from(value: SignedSatoshis) -> Result<Self, Self::Error> {
        if value.0 < 0 {
            Err(ConversionError::UnexpectedNegativeNumber(value.to_btc()))
        } else {
            Ok(Satoshis(value.0 as u64))
        }
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }

    // Extract two 64-bit values from the input
    let a_bytes: [u8; 8] = data[0..8].try_into().unwrap();
    let b_bytes: [u8; 8] = data[8..16].try_into().unwrap();
    
    let a_u64 = u64::from_le_bytes(a_bytes);
    let b_u64 = u64::from_le_bytes(b_bytes);
    let a_i64 = i64::from_le_bytes(a_bytes);
    let b_i64 = i64::from_le_bytes(b_bytes);

    // Test unsigned Satoshis operations
    test_unsigned_satoshis(a_u64, b_u64);
    
    // Test signed Satoshis operations  
    test_signed_satoshis(a_i64, b_i64);
    
    // Test conversions between signed and unsigned
    test_conversions(a_u64, a_i64);
    
    // Test BTC conversions with reasonable values
    if let Some(reasonable_decimal) = make_reasonable_decimal(a_u64) {
        test_btc_conversions(reasonable_decimal);
    }
});

fn test_unsigned_satoshis(a: u64, b: u64) {
    let sats_a = Satoshis::from(a);
    let sats_b = Satoshis::from(b);
    
    // Test basic operations that shouldn't panic
    let _ = sats_a.to_btc();
    let _ = sats_a.into_inner();
    let _ = sats_a.formatted_btc();
    let _ = sats_a.is_zero();
    
    // Test arithmetic operations with overflow protection
    if let Some(sum) = a.checked_add(b) {
        let result = sats_a + sats_b;
        assert_eq!(result.into_inner(), sum);
    }
    
    if a >= b {
        let result = sats_a - sats_b;
        assert_eq!(result.into_inner(), a - b);
    }
    
    // Test assignment operations
    let mut sats_mut = sats_a;
    if let Some(_) = a.checked_add(b) {
        sats_mut += sats_b;
    }
    
    let mut sats_mut = sats_a;
    if a >= b {
        sats_mut -= sats_b;
    }
}

fn test_signed_satoshis(a: i64, b: i64) {
    let signed_a = SignedSatoshis::from(Satoshis::from(a.unsigned_abs()));
    let signed_b = SignedSatoshis::from(Satoshis::from(b.unsigned_abs()));
    
    // Create proper signed values
    let signed_a = if a >= 0 { signed_a } else { SignedSatoshis::ZERO - signed_a };
    let signed_b = if b >= 0 { signed_b } else { SignedSatoshis::ZERO - signed_b };
    
    // Test basic operations
    let _ = signed_a.to_btc();
    let _ = signed_a.into_inner();
    let _ = signed_a.abs();
    
    // Test arithmetic operations with overflow protection
    if let Some(sum) = a.checked_add(b) {
        let result = signed_a + signed_b;
        assert_eq!(result.into_inner(), sum);
    }
    
    if let Some(diff) = a.checked_sub(b) {
        let result = signed_a - signed_b;
        assert_eq!(result.into_inner(), diff);
    }
}

fn test_conversions(a_u64: u64, a_i64: i64) {
    // Test conversion from unsigned to signed
    let sats = Satoshis::from(a_u64);
    if a_u64 <= i64::MAX as u64 {
        let signed_result = SignedSatoshis::from(sats);
        assert_eq!(signed_result.into_inner(), a_u64 as i64);
    }
    
    // Test conversion from signed to unsigned (when non-negative)
    if a_i64 >= 0 {
        let signed_sats = SignedSatoshis::from(Satoshis::from(a_i64.unsigned_abs()));
        match signed_sats.try_into() {
            Ok(unsigned_result) => {
                let result: Satoshis = unsigned_result;
                assert_eq!(result.into_inner(), a_i64 as u64);
            }
            Err(_) => {
                // Expected for negative values
            }
        }
    }
}

fn test_btc_conversions(btc_decimal: Decimal) {
    // Test unsigned Satoshis BTC conversion
    match Satoshis::try_from_btc(btc_decimal) {
        Ok(sats) => {
            let back_to_btc = sats.to_btc();
            // Allow for small precision differences due to decimal arithmetic
            let diff = (back_to_btc - btc_decimal).abs();
            assert!(diff < Decimal::new(1, 8)); // Less than 1 satoshi
        }
        Err(ConversionError::UnexpectedNegativeNumber(_)) => {
            // Expected for negative values
        }
        Err(ConversionError::DecimalError) => {
            // Expected for out-of-range values
        }
    }
    
    // Test signed Satoshis BTC conversion (only for reasonable values)
    if btc_decimal.abs() < Decimal::new(21_000_000, 0) { // Less than total BTC supply
        // Only test from_btc for values that won't overflow i64
        if btc_decimal.abs() < Decimal::new(92_233_720_368, 0) { // ~92B BTC (way more than exists)
            // Use a try-catch equivalent since from_btc can panic
            let signed_result = std::panic::catch_unwind(|| {
                SignedSatoshis::from_btc(btc_decimal)
            });
            
            if let Ok(signed_sats) = signed_result {
                let back_to_btc = signed_sats.to_btc();
                let diff = (back_to_btc - btc_decimal).abs();
                assert!(diff < Decimal::new(1, 8)); // Less than 1 satoshi
            }
        }
    }
}

fn make_reasonable_decimal(input: u64) -> Option<Decimal> {
    // Create a reasonable decimal value for BTC conversion testing
    // Limit to reasonable BTC amounts to avoid overflow issues
    
    // Scale down large numbers to reasonable BTC amounts
    let scaled = if input > 21_000_000_000_000_000 {
        input % 21_000_000_000_000_000 // Mod by 21M BTC in satoshis
    } else {
        input
    };
    
    // Convert to BTC (divide by 100M satoshis per BTC)
    let btc_amount = Decimal::new(scaled as i64, 8);
    
    // Sometimes make it negative for signed testing
    if (input % 2) == 0 {
        Some(-btc_amount)
    } else {
        Some(btc_amount)
    }
}