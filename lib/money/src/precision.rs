use crate::error::ConversionError;

/// Validated precision for regulatory rounding boundaries.
///
/// Represents a decimal-places count in the range 2..=28.
/// Used only at explicit rounding boundaries (e.g., `round_to_precision`),
/// not carried on every `CalculationAmount`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Precision(u32);

impl Precision {
    const MIN_DP: u32 = 2;
    const MAX_DP: u32 = 28;

    pub fn try_new(dp: u32) -> Result<Self, ConversionError> {
        if !(Self::MIN_DP..=Self::MAX_DP).contains(&dp) {
            return Err(ConversionError::InvalidPrecision(dp));
        }
        Ok(Self(dp))
    }

    pub fn dp(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_precision() {
        assert!(Precision::try_new(2).is_ok());
        assert!(Precision::try_new(6).is_ok());
        assert!(Precision::try_new(28).is_ok());
    }

    #[test]
    fn precision_too_low() {
        assert!(matches!(
            Precision::try_new(0),
            Err(ConversionError::InvalidPrecision(0))
        ));
        assert!(matches!(
            Precision::try_new(1),
            Err(ConversionError::InvalidPrecision(1))
        ));
    }

    #[test]
    fn precision_too_high() {
        assert!(matches!(
            Precision::try_new(29),
            Err(ConversionError::InvalidPrecision(29))
        ));
    }
}
