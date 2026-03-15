use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PriceProviderConfig {
    Bitfinex,
}
