use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::PriceOfOneBTC;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum CorePriceEvent {
    BtcUsdPriceUpdated {
        price: PriceOfOneBTC,
        recorded_at: DateTime<Utc>,
    },
}
