use domain_config::define_internal_config;
use rust_decimal::RoundingStrategy;
use serde::{Deserialize, Serialize};

define_internal_config! {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub(crate) struct RoundingPolicyConfig {
        pub(crate) lender_favorable: RoundingStrategyConfig,
        pub(crate) conservative: RoundingStrategyConfig,
    }

    spec {
        key: "credit-rounding-policy";
        default: || Some(RoundingPolicyConfig {
            lender_favorable: RoundingStrategyConfig::AwayFromZero,
            conservative: RoundingStrategyConfig::ToZero,
        });
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum RoundingStrategyConfig {
    AwayFromZero,
    ToZero,
    MidpointNearestEven,
}

impl RoundingStrategyConfig {
    pub(crate) fn to_rust_decimal(&self) -> RoundingStrategy {
        match self {
            Self::AwayFromZero => RoundingStrategy::AwayFromZero,
            Self::ToZero => RoundingStrategy::ToZero,
            Self::MidpointNearestEven => RoundingStrategy::MidpointNearestEven,
        }
    }
}
