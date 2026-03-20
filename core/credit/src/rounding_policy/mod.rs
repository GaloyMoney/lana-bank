use domain_config::{DomainConfigError, define_exposed_config};
use rust_decimal::RoundingStrategy;
use serde::{Deserialize, Serialize};

define_exposed_config! {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub(crate) struct AccrualPrecisionDp(u64);

    spec {
        key: "credit-accrual-precision-dp";
        validate: |value: &u64| {
            if *value < 2 {
                return Err(DomainConfigError::InvalidState(
                    "accrual precision must be at least 2 decimal places".to_string(),
                ));
            }
            if *value > 28 {
                return Err(DomainConfigError::InvalidState(
                    "accrual precision cannot exceed 28 decimal places".to_string(),
                ));
            }
            Ok(())
        };
    }
}

define_exposed_config! {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub(crate) struct AccrualRoundingStrategy(String);

    spec {
        key: "credit-accrual-rounding-strategy";
        validate: |value: &String| {
            parse_rounding_strategy(value).map(|_| ())
        };
    }
}

pub(crate) fn parse_rounding_strategy(s: &str) -> Result<RoundingStrategy, DomainConfigError> {
    match s {
        "away_from_zero" => Ok(RoundingStrategy::AwayFromZero),
        "to_zero" => Ok(RoundingStrategy::ToZero),
        "midpoint_away_from_zero" => Ok(RoundingStrategy::MidpointAwayFromZero),
        _ => Err(DomainConfigError::InvalidState(format!(
            "invalid rounding strategy '{}'. Must be one of: away_from_zero, to_zero, midpoint_away_from_zero",
            s
        ))),
    }
}
