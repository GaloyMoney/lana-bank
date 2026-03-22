use std::str::FromStr;

use crate::error::ConversionError;

/// Validated rounding mode for financial calculations.
///
/// Represents the subset of `rust_decimal::RoundingStrategy` values
/// that the domain supports for configurable rounding behavior.
/// Uses snake_case string representation for config storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum RoundingMode {
    AwayFromZero,
    ToZero,
    MidpointAwayFromZero,
}

impl RoundingMode {
    pub fn try_from_str(s: &str) -> Result<Self, ConversionError> {
        Self::from_str(s).map_err(|_| ConversionError::InvalidRoundingMode(s.to_owned()))
    }
}

impl From<RoundingMode> for rust_decimal::RoundingStrategy {
    fn from(mode: RoundingMode) -> Self {
        match mode {
            RoundingMode::AwayFromZero => Self::AwayFromZero,
            RoundingMode::ToZero => Self::ToZero,
            RoundingMode::MidpointAwayFromZero => Self::MidpointAwayFromZero,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_modes_round_trip() {
        for (s, expected) in [
            ("away_from_zero", RoundingMode::AwayFromZero),
            ("to_zero", RoundingMode::ToZero),
            (
                "midpoint_away_from_zero",
                RoundingMode::MidpointAwayFromZero,
            ),
        ] {
            let parsed = RoundingMode::try_from_str(s).unwrap();
            assert_eq!(parsed, expected);
            assert_eq!(parsed.to_string(), s);
        }
    }

    #[test]
    fn invalid_mode_returns_error() {
        assert!(matches!(
            RoundingMode::try_from_str("bankers"),
            Err(ConversionError::InvalidRoundingMode(s)) if s == "bankers"
        ));
    }

    #[test]
    fn converts_to_rust_decimal() {
        let rs: rust_decimal::RoundingStrategy = RoundingMode::AwayFromZero.into();
        assert_eq!(rs, rust_decimal::RoundingStrategy::AwayFromZero);
    }
}
