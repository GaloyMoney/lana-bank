use chrono::{DateTime, Utc};
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::primitives::PriceOfOneBTC;
use obix::out::EphemeralEventType;

pub const PRICE_UPDATED_EVENT_TYPE: EphemeralEventType =
    EphemeralEventType::new("core.price.price-updated");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CorePriceEvent {
    PriceUpdated {
        price: PriceOfOneBTC,
        timestamp: DateTime<Utc>,
    },
}
