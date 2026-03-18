use domain_config::{DomainConfigError, define_exposed_config};
use serde::{Deserialize, Serialize};

define_exposed_config! {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub(crate) struct AccrualPrecisionConfig(u64);

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
                    "accrual precision cannot exceed 28 decimal places (Decimal limit)".to_string(),
                ));
            }
            Ok(())
        };
    }
}
