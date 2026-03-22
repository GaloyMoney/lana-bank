use domain_config::{DomainConfigError, define_exposed_config};
use money::Precision;
use rust_decimal::RoundingStrategy;
use serde::{Deserialize, Serialize};

const VALID_STRATEGIES: &[&str] = &["away_from_zero", "to_zero", "midpoint_away_from_zero"];

define_exposed_config! {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct AccrualPrecisionDp(u64);

    spec {
        key: "credit-accrual-precision-dp";
        validate: |value: &u64| {
            Precision::try_new(*value as u32)
                .map(|_| ())
                .map_err(|e| DomainConfigError::InvalidState(format!("invalid accrual precision: {e}")))
        };
    }
}

define_exposed_config! {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct AccrualRoundingStrategy(String);

    spec {
        key: "credit-accrual-rounding-strategy";
        validate: |value: &String| {
            if VALID_STRATEGIES.contains(&value.as_str()) {
                Ok(())
            } else {
                Err(DomainConfigError::InvalidState(format!(
                    "invalid rounding strategy '{}'. Must be one of: {}",
                    value,
                    VALID_STRATEGIES.join(", ")
                )))
            }
        };
    }
}

/// Converts a validated rounding strategy string into a `RoundingStrategy`.
///
/// # Panics
///
/// Panics if the string is not one of the accepted values. This is safe because
/// the domain config `validate` function already rejects invalid values at write-time.
pub(crate) fn parse_rounding_strategy(s: &str) -> RoundingStrategy {
    match s {
        "away_from_zero" => RoundingStrategy::AwayFromZero,
        "to_zero" => RoundingStrategy::ToZero,
        "midpoint_away_from_zero" => RoundingStrategy::MidpointAwayFromZero,
        _ => unreachable!(
            "invalid rounding strategy '{}' — domain config validation should have rejected this",
            s
        ),
    }
}
