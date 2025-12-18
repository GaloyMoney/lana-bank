use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeEventsAction;

impl TimeEventsAction {
    pub const BROADCAST_DAILY_CLOSING: Self = Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeEventsObject;

impl TimeEventsObject {
    pub fn daily_closing() -> Self {
        Self
    }
}
