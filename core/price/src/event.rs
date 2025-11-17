use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::primitives::PriceOfOneBTC;

pub const PRICE_UPDATED_EVENT_TYPE: &str = "core.price.price-updated";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CorePriceEvent {
    PriceUpdated { price: PriceOfOneBTC },
}
