use async_graphql::SimpleObject;

use crate::primitives::Timestamp;

#[derive(SimpleObject)]
pub struct ServerTime {
    current_time: Timestamp,
    is_artificial: bool,
}

impl ServerTime {
    pub fn new(current_time: Timestamp, is_artificial: bool) -> Self {
        Self {
            current_time,
            is_artificial,
        }
    }
}
