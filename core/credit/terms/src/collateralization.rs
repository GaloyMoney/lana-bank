use core_price::PriceOfOneBTC;
use money::Satoshis;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CollateralizationRatio {
    Finite(Decimal),
    Infinite,
}

impl Default for CollateralizationRatio {
    fn default() -> Self {
        Self::Finite(Decimal::ZERO)
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
    Eq,
    strum::Display,
    strum::EnumString,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CollateralizationState {
    FullyCollateralized,
    UnderMarginCallThreshold,
    UnderLiquidationThreshold,
    #[default]
    NoCollateral,
    NoExposure,
}

impl CollateralizationState {
    pub const fn is_under_liquidation_threshold(&self) -> bool {
        matches!(self, Self::UnderLiquidationThreshold)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize, Eq)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum PendingCreditFacilityCollateralizationState {
    FullyCollateralized {
        collateral: Satoshis,
        price: PriceOfOneBTC,
    },
    UnderCollateralized {
        collateral: Satoshis,
        price: PriceOfOneBTC,
    },
    #[default]
    NoCollateral,
}

impl PendingCreditFacilityCollateralizationState {
    pub fn is_fully_collateralized(&self) -> bool {
        matches!(self, Self::FullyCollateralized { .. })
    }

    pub fn is_same_variant(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

// SQLx implementations for database storage
mod collateralization_state_sqlx {
    use sqlx::{Type, postgres::*};

    use super::CollateralizationState;

    impl Type<Postgres> for CollateralizationState {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for CollateralizationState {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for CollateralizationState {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for CollateralizationState {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}

mod collateralization_ratio_sqlx {
    use rust_decimal::Decimal;
    use sqlx::{Type, postgres::*};

    use super::CollateralizationRatio;

    impl Type<Postgres> for CollateralizationRatio {
        fn type_info() -> PgTypeInfo {
            <Option<Decimal> as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <Option<Decimal> as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for CollateralizationRatio {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            let opt: Option<Decimal> = match *self {
                CollateralizationRatio::Finite(d) => Some(d),
                CollateralizationRatio::Infinite => None,
            };
            <Option<Decimal> as sqlx::Encode<'_, Postgres>>::encode(opt, buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for CollateralizationRatio {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let opt: Option<Decimal> = <Option<Decimal> as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(match opt {
                Some(d) => CollateralizationRatio::Finite(d),
                None => CollateralizationRatio::Infinite,
            })
        }
    }

    impl PgHasArrayType for CollateralizationRatio {
        fn array_type_info() -> PgTypeInfo {
            <Option<Decimal> as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}

mod pending_collateralization_state_sqlx {
    use core_price::PriceOfOneBTC;
    use money::Satoshis;
    use sqlx::{Type, postgres::*};

    use super::PendingCreditFacilityCollateralizationState;

    const FULLY_COLLATERALIZED: &str = "FullyCollateralized";
    const UNDER_COLLATERALIZED: &str = "UnderCollateralized";
    const NO_COLLATERAL: &str = "NoCollateral";

    impl Type<Postgres> for PendingCreditFacilityCollateralizationState {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for PendingCreditFacilityCollateralizationState {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            let s = match self {
                PendingCreditFacilityCollateralizationState::FullyCollateralized { .. } => {
                    FULLY_COLLATERALIZED
                }
                PendingCreditFacilityCollateralizationState::UnderCollateralized { .. } => {
                    UNDER_COLLATERALIZED
                }
                PendingCreditFacilityCollateralizationState::NoCollateral => NO_COLLATERAL,
            };
            <String as sqlx::Encode<'_, Postgres>>::encode(s.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for PendingCreditFacilityCollateralizationState {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            match s.as_str() {
                FULLY_COLLATERALIZED => Ok(
                    PendingCreditFacilityCollateralizationState::FullyCollateralized {
                        collateral: Satoshis::ZERO,
                        price: PriceOfOneBTC::ZERO,
                    },
                ),
                UNDER_COLLATERALIZED => Ok(
                    PendingCreditFacilityCollateralizationState::UnderCollateralized {
                        collateral: Satoshis::ZERO,
                        price: PriceOfOneBTC::ZERO,
                    },
                ),
                NO_COLLATERAL => Ok(PendingCreditFacilityCollateralizationState::NoCollateral),
                other => Err(format!(
                    "Unknown PendingCreditFacilityCollateralizationState: {other}"
                )
                .into()),
            }
        }
    }

    impl PgHasArrayType for PendingCreditFacilityCollateralizationState {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}
