use crate::PriceOfOneBTC;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PriceUpdated {
    pub price: PriceOfOneBTC,
}
