use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::primitives::PriceOfOneBTC;
use outbox::EphemeralEventType;

pub const PRICE_UPDATED_EVENT_TYPE: EphemeralEventType =
    EphemeralEventType::from_static("core.price.price-updated");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CorePriceEvent {
    PriceUpdated { price: PriceOfOneBTC },
}
