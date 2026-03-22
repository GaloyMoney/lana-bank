use domain_config::{DomainConfigError, define_exposed_config};
use money::{Precision, RoundingMode};
use serde::{Deserialize, Serialize};

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
            RoundingMode::try_from_str(value)
                .map(|_| ())
                .map_err(|e| DomainConfigError::InvalidState(format!("{e}")))
        };
    }
}
