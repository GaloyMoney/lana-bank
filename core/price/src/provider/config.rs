use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, serde::Serialize, serde::Deserialize))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
#[strum_discriminants(serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "json-schema",
    strum_discriminants(derive(schemars::JsonSchema))
)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PriceProviderConfig {
    Bitfinex,
    ManualPrice { usd_cents_per_btc: u64 },
}
